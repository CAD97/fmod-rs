//! This is a very simple test that releasing an existing system returns the
//! global lock to a state where creating a new system is possible safely.
//!
//! It additionally would serve as an automatable check example...
//! if there were a way to download FMOD's runtime from CI simply.
//! (Self hosting is *very* iffy legally, and there's no binary blob secrets.)

fn main() -> anyhow::Result<()> {
    {
        let system = fmod::System::new()?;
        system.init(32, fmod::InitFlags::Normal)?;
    }

    {
        let system = fmod::System::new()?;
        system.init(32, fmod::InitFlags::Normal)?;
    }

    Ok(())
}
