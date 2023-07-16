use fmod::*;

pub trait PcmCallback {
    fn read(sound: &Sound, data: &mut [u8]) -> Result;
    fn seek(sound: &Sound, subsound: i32, position: Time) -> Result;
}

pub trait NonBlockCallback {
    fn notify(sound: &Sound, result: Result) -> Result;
}
