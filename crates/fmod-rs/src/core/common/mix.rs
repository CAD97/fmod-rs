use {
    fmod::utils::{Const, LessThanOrEqual},
    fmod::*,
    std::{
        ops::{Index, IndexMut},
        ptr::{self, NonNull},
        slice,
    },
};

/// Types which can be viewed as a mix matrix.
pub trait AsMixMatrix {
    /// View this type as a mix matrix.
    fn as_mix_matrix(&self) -> &MixMatrix;
    /// View this type mutably as a mix matrix.
    fn as_mix_matrix_mut(&mut self) -> &mut MixMatrix;
}

impl<const IN: usize, const OUT: usize> AsMixMatrix for [[f32; IN]; OUT]
where
    Const<IN>: LessThanOrEqual<MAX_CHANNEL_WIDTH>,
    Const<OUT>: LessThanOrEqual<MAX_CHANNEL_WIDTH>,
{
    fn as_mix_matrix(&self) -> &MixMatrix {
        MixMatrix::new_ref(slice_flatten(self), IN, OUT)
    }

    fn as_mix_matrix_mut(&mut self) -> &mut MixMatrix {
        MixMatrix::new_mut(slice_flatten_mut(self), IN, OUT)
    }
}

impl AsMixMatrix for MixMatrix {
    fn as_mix_matrix(&self) -> &MixMatrix {
        self
    }

    fn as_mix_matrix_mut(&mut self) -> &mut MixMatrix {
        self
    }
}

/// The maximum supported size for a mix matrix. Caution: 4KiB.
pub type MaxMixMatrix = [[f32; MAX_CHANNEL_WIDTH]; MAX_CHANNEL_WIDTH];

/// A mix matrix large enough to mix between any two standard [`SpeakerMode`].
pub type SurroundMixMatrix = [[f32; 12]; 12];

/// A 2 dimensional mix matrix that maps the signal from
/// input channels (columns) to output speakers (rows).
///
/// Indexed as `matrix[(in_channel, out_channel)]`, either with channel indices
/// or with the [`Speaker`] enum. Indexing with [`Speaker`] assumes that the
/// matrix mixes typical [`SpeakerMode`]s and will panic if not sized for such.
///
/// Permits striding a smaller valid region inside a larger matrix, i.e. that
/// offsetting from one output channel (row) to the next is a larger step than
/// just the data actually used for that output channel.
#[repr(transparent)]
pub struct MixMatrix([()]);

impl MixMatrix {
    pub(crate) fn new(matrix: *mut f32, in_channels: usize, out_channels: usize) -> *mut Self {
        assert!(in_channels <= MAX_CHANNEL_WIDTH);
        assert!(out_channels <= MAX_CHANNEL_WIDTH);
        let data = ptr_launder(matrix) as *mut ();
        let len = out_channels | (in_channels << 8) | (out_channels << 16) | (in_channels << 24);
        ptr::slice_from_raw_parts_mut(data, len) as *mut Self
    }

    pub(crate) fn new_ref(matrix: &[f32], in_channels: usize, out_channels: usize) -> &Self {
        let _ = &matrix[..in_channels * out_channels];
        unsafe { &*Self::new(matrix.as_ptr().cast_mut(), in_channels, out_channels) }
    }

    pub(crate) fn new_mut(
        matrix: &mut [f32],
        in_channels: usize,
        out_channels: usize,
    ) -> &mut Self {
        let _ = &matrix[..in_channels * out_channels];
        unsafe { &mut *Self::new(matrix.as_mut_ptr(), in_channels, out_channels) }
    }

    pub(crate) fn matrix(&self) -> &[f32] {
        let len = self.out_channel_hop() * self.in_channel_hop();
        let data = ptr_launder(ptr_from_ref(self).cast());
        unsafe { slice::from_raw_parts(data, len) }
    }

    pub(crate) fn matrix_mut(&mut self) -> &mut [f32] {
        let len = self.out_channel_hop() * self.in_channel_hop();
        let data = ptr_launder(ptr_from_mut(self).cast());
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    /// The number of output channels (rows) in the matrix.
    pub fn out_channels(&self) -> usize {
        self.0.len() & 0xFF
    }

    /// The number of input channels (columns) in the matrix.
    pub fn in_channels(&self) -> usize {
        (self.0.len() >> 8) & 0xFF
    }

    /// The full number of output channels (rows) accessible in the matrix,
    /// including those sliced off.
    pub(crate) fn out_channel_hop(&self) -> usize {
        (self.0.len() >> 16) & 0xFF
    }

    /// The full number of input channels (columns) accessible in the matrix,
    /// including those strode over.
    pub(crate) fn in_channel_hop(&self) -> usize {
        (self.0.len() >> 24) & 0xFF
    }

    /// A ZST slice to index for output channel index bounds checking.
    fn out_channel_phantom(&self) -> &[()] {
        let len = self.out_channels();
        let data = NonNull::dangling().as_ptr();
        unsafe { slice::from_raw_parts(data, len) }
    }

    /// A ZST slice to index for input channel index bounds checking.
    fn in_channel_phantom(&self) -> &[()] {
        let len = self.in_channels();
        let data = NonNull::dangling().as_ptr();
        unsafe { slice::from_raw_parts(data, len) }
    }

    /// Slice this matrix to a smaller valid region within the larger matrix.
    pub fn slice(&self, in_channels: usize, out_channels: usize) -> &Self {
        let _ = &self.in_channel_phantom()[..in_channels];
        let _ = &self.out_channel_phantom()[..out_channels];
        let data = self.0.as_ptr();
        let len = self.0.len() & 0xFFFF_0000 | out_channels | (in_channels << 8);
        unsafe { &*(ptr::slice_from_raw_parts(data, len) as *const Self) }
    }

    /// Slice this matrix to a smaller valid region within the larger matrix.
    pub fn slice_mut(&mut self, in_channels: usize, out_channels: usize) -> &mut Self {
        let _ = &self.in_channel_phantom()[..in_channels];
        let _ = &self.out_channel_phantom()[..out_channels];
        let data = self.0.as_mut_ptr();
        let len = self.0.len() & 0xFFFF_0000 | out_channels | (in_channels << 8);
        unsafe { &mut *(ptr::slice_from_raw_parts_mut(data, len) as *mut Self) }
    }
}

impl Index<(usize, usize)> for MixMatrix {
    type Output = f32;

    fn index(&self, (in_channel, out_channel): (usize, usize)) -> &f32 {
        self.in_channel_phantom()[in_channel];
        self.out_channel_phantom()[out_channel];
        let in_channel_hop = self.in_channel_hop();
        &self.matrix()[out_channel * in_channel_hop + in_channel]
    }
}

impl IndexMut<(usize, usize)> for MixMatrix {
    fn index_mut(&mut self, (in_channel, out_channel): (usize, usize)) -> &mut f32 {
        self.in_channel_phantom()[in_channel];
        self.out_channel_phantom()[out_channel];
        let in_channel_hop = self.in_channel_hop();
        &mut self.matrix_mut()[out_channel * in_channel_hop + in_channel]
    }
}

const fn channel_count_to_mode(count: usize) -> SpeakerMode {
    match count {
        1 => SpeakerMode::Mono,
        2 => SpeakerMode::Stereo,
        4 => SpeakerMode::Quad,
        5 => SpeakerMode::Surround,
        6 => SpeakerMode::Surround51,
        8 => SpeakerMode::Surround71,
        12 => SpeakerMode::Surround714,
        _ => SpeakerMode::Raw,
    }
}

fn speaker_to_channel(index: Speaker, mode: SpeakerMode) -> usize {
    match (mode, index) {
        (SpeakerMode::Mono, Speaker::FrontCenter) => 0,

        (SpeakerMode::Stereo, Speaker::FrontLeft) => 0,
        (SpeakerMode::Stereo, Speaker::FrontRight) => 1,

        (SpeakerMode::Quad, Speaker::FrontLeft) => 0,
        (SpeakerMode::Quad, Speaker::FrontRight) => 1,
        (SpeakerMode::Quad, Speaker::SurroundLeft) => 2,
        (SpeakerMode::Quad, Speaker::SurroundRight) => 3,

        (SpeakerMode::Surround, Speaker::FrontLeft) => 0,
        (SpeakerMode::Surround, Speaker::FrontRight) => 1,
        (SpeakerMode::Surround, Speaker::FrontCenter) => 2,
        (SpeakerMode::Surround, Speaker::SurroundLeft) => 3,
        (SpeakerMode::Surround, Speaker::SurroundRight) => 4,

        (SpeakerMode::Surround51, Speaker::FrontLeft) => 0,
        (SpeakerMode::Surround51, Speaker::FrontRight) => 1,
        (SpeakerMode::Surround51, Speaker::FrontCenter) => 2,
        (SpeakerMode::Surround51, Speaker::LowFrequency) => 3,
        (SpeakerMode::Surround51, Speaker::SurroundLeft) => 4,
        (SpeakerMode::Surround51, Speaker::SurroundRight) => 5,

        (SpeakerMode::Surround71, Speaker::FrontLeft) => 0,
        (SpeakerMode::Surround71, Speaker::FrontRight) => 1,
        (SpeakerMode::Surround71, Speaker::FrontCenter) => 2,
        (SpeakerMode::Surround71, Speaker::LowFrequency) => 3,
        (SpeakerMode::Surround71, Speaker::SurroundLeft) => 4,
        (SpeakerMode::Surround71, Speaker::SurroundRight) => 5,
        (SpeakerMode::Surround71, Speaker::BackLeft) => 6,
        (SpeakerMode::Surround71, Speaker::BackRight) => 7,

        (SpeakerMode::Surround714, Speaker::FrontLeft) => 0,
        (SpeakerMode::Surround714, Speaker::FrontRight) => 1,
        (SpeakerMode::Surround714, Speaker::FrontCenter) => 2,
        (SpeakerMode::Surround714, Speaker::LowFrequency) => 3,
        (SpeakerMode::Surround714, Speaker::SurroundLeft) => 4,
        (SpeakerMode::Surround714, Speaker::SurroundRight) => 5,
        (SpeakerMode::Surround714, Speaker::BackLeft) => 6,
        (SpeakerMode::Surround714, Speaker::BackRight) => 7,
        (SpeakerMode::Surround714, Speaker::TopFrontLeft) => 8,
        (SpeakerMode::Surround714, Speaker::TopFrontRight) => 9,
        (SpeakerMode::Surround714, Speaker::TopBackLeft) => 10,
        (SpeakerMode::Surround714, Speaker::TopBackRight) => 11,

        (mode, index) => panic!("{index:?} is an invalid speaker index for {mode:?}"),
    }
}

impl Index<(Speaker, Speaker)> for MixMatrix {
    type Output = f32;

    fn index(&self, (in_speaker, out_speaker): (Speaker, Speaker)) -> &f32 {
        let in_speaker_mode = channel_count_to_mode(self.in_channels());
        let out_speaker_mode = channel_count_to_mode(self.out_channels());
        let in_channel = speaker_to_channel(in_speaker, in_speaker_mode);
        let out_channel = speaker_to_channel(out_speaker, out_speaker_mode);
        &self[(in_channel, out_channel)]
    }
}

impl IndexMut<(Speaker, Speaker)> for MixMatrix {
    fn index_mut(&mut self, (in_speaker, out_speaker): (Speaker, Speaker)) -> &mut f32 {
        let in_speaker_mode = channel_count_to_mode(self.in_channels());
        let out_speaker_mode = channel_count_to_mode(self.out_channels());
        let in_channel = speaker_to_channel(in_speaker, in_speaker_mode);
        let out_channel = speaker_to_channel(out_speaker, out_speaker_mode);
        &mut self[(in_channel, out_channel)]
    }
}

impl Index<(usize, Speaker)> for MixMatrix {
    type Output = f32;

    fn index(&self, (in_channel, out_speaker): (usize, Speaker)) -> &f32 {
        let out_speaker_mode = channel_count_to_mode(self.out_channels());
        let out_channel = speaker_to_channel(out_speaker, out_speaker_mode);
        &self[(in_channel, out_channel)]
    }
}

impl IndexMut<(usize, Speaker)> for MixMatrix {
    fn index_mut(&mut self, (in_channel, out_speaker): (usize, Speaker)) -> &mut f32 {
        let out_speaker_mode = channel_count_to_mode(self.out_channels());
        let out_channel = speaker_to_channel(out_speaker, out_speaker_mode);
        &mut self[(in_channel, out_channel)]
    }
}

impl Index<(Speaker, usize)> for MixMatrix {
    type Output = f32;

    fn index(&self, (in_speaker, out_channel): (Speaker, usize)) -> &f32 {
        let in_speaker_mode = channel_count_to_mode(self.in_channels());
        let in_channel = speaker_to_channel(in_speaker, in_speaker_mode);
        &self[(in_channel, out_channel)]
    }
}

impl IndexMut<(Speaker, usize)> for MixMatrix {
    fn index_mut(&mut self, (in_speaker, out_channel): (Speaker, usize)) -> &mut f32 {
        let in_speaker_mode = channel_count_to_mode(self.in_channels());
        let in_channel = speaker_to_channel(in_speaker, in_speaker_mode);
        &mut self[(in_channel, out_channel)]
    }
}

const fn ptr_from_ref<T: ?Sized>(r: &T) -> *const T {
    r
}

fn ptr_from_mut<T: ?Sized>(r: &mut T) -> *mut T {
    r
}

/// pointer crimes go here
fn ptr_launder<T>(p: *const T) -> *mut T {
    p as usize as *mut T
}

fn slice_flatten<T, const N: usize>(s: &[[T; N]]) -> &[T] {
    let len = s.len() * N;
    let data = s.as_ptr().cast();
    unsafe { slice::from_raw_parts(data, len) }
}

fn slice_flatten_mut<T, const N: usize>(s: &mut [[T; N]]) -> &mut [T] {
    let len = s.len() * N;
    let data = s.as_mut_ptr().cast();
    unsafe { slice::from_raw_parts_mut(data, len) }
}
