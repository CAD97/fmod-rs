use {fmod::raw::*, fmod_example_framework::*, std::ptr};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = ptr::null_mut();
    let [mut sound1, mut sound2, mut sound3] = [ptr::null_mut(); 3];
    let mut channel = ptr::null_mut();
    let mut result;
    let extra_driver_data = ptr::null_mut();

    init()?;

    unsafe {
        result = FMOD_System_Create(&mut system, FMOD_VERSION);
        errcheck!(result);

        result = FMOD_System_Init(system, 32, FMOD_INIT_NORMAL, extra_driver_data);
        errcheck!(result);

        result = FMOD_System_CreateSound(
            system,
            media_path("drumloop.wav"),
            FMOD_DEFAULT,
            ptr::null_mut(),
            &mut sound1,
        );
        errcheck!(result);

        // drumloop.wav has embedded loop points which automatically makes looping turn on,
        // so turn it off here.  We could have also just put FMOD_LOOP_OFF in the above CreateSound call.
        result = FMOD_Sound_SetMode(sound1, FMOD_LOOP_OFF);
        errcheck!(result);

        result = FMOD_System_CreateSound(
            system,
            media_path("jaguar.wav"),
            FMOD_DEFAULT,
            ptr::null_mut(),
            &mut sound2,
        );
        errcheck!(result);

        result = FMOD_System_CreateSound(
            system,
            media_path("swish.wav"),
            FMOD_DEFAULT,
            ptr::null_mut(),
            &mut sound3,
        );
        errcheck!(result);

        while !btn_press(Buttons::Quit) {
            update()?;
            if btn_press(Buttons::Action1) {
                result = FMOD_System_PlaySound(
                    system,
                    sound1,
                    ptr::null_mut(),
                    false as _,
                    &mut channel,
                );
                errcheck!(result);
            }
            if btn_press(Buttons::Action2) {
                result = FMOD_System_PlaySound(
                    system,
                    sound2,
                    ptr::null_mut(),
                    false as _,
                    &mut channel,
                );
                errcheck!(result);
            }
            if btn_press(Buttons::Action3) {
                result = FMOD_System_PlaySound(
                    system,
                    sound3,
                    ptr::null_mut(),
                    false as _,
                    &mut channel,
                );
                errcheck!(result);
            }
            result = FMOD_System_Update(system);
            errcheck!(result);
            {
                let mut ms = 0;
                let mut lenms = 0;
                let mut playing = 0;
                let mut paused = 0;
                let mut channelsplaying = 0;

                if !channel.is_null() {
                    let mut currentsound = ptr::null_mut();
                    result = FMOD_Channel_IsPlaying(channel, &mut playing);
                    if result != FMOD_OK
                        && result != FMOD_ERR_INVALID_HANDLE
                        && result != FMOD_ERR_CHANNEL_STOLEN
                    {
                        errcheck!(result)
                    }
                    result = FMOD_Channel_GetPaused(channel, &mut paused);
                    if result != FMOD_OK
                        && result != FMOD_ERR_INVALID_HANDLE
                        && result != FMOD_ERR_CHANNEL_STOLEN
                    {
                        errcheck!(result)
                    }
                    result = FMOD_Channel_GetPosition(channel, &mut ms, FMOD_TIMEUNIT_MS);
                    if result != FMOD_OK
                        && result != FMOD_ERR_INVALID_HANDLE
                        && result != FMOD_ERR_CHANNEL_STOLEN
                    {
                        errcheck!(result)
                    }
                    FMOD_Channel_GetCurrentSound(channel, &mut currentsound);
                    if !currentsound.is_null() {
                        result = FMOD_Sound_GetLength(currentsound, &mut lenms, FMOD_TIMEUNIT_MS);
                        if result != FMOD_OK
                            && result != FMOD_ERR_INVALID_HANDLE
                            && result != FMOD_ERR_CHANNEL_STOLEN
                        {
                            errcheck!(result)
                        }
                    }
                }

                FMOD_System_GetChannelsPlaying(system, &mut channelsplaying, ptr::null_mut());

                draw(format_args!(
                    "=================================================="
                ));
                draw(format_args!("Play Sound Example."));
                draw(format_args!(
                    "Copyright (c) Firelight Technologies 2004-2021."
                ));
                draw(format_args!(
                    "=================================================="
                ));
                draw(format_args!(""));
                draw(format_args!(
                    "Press {} to play a mono sound (drumloop)",
                    btn_str(Buttons::Action1),
                ));
                draw(format_args!(
                    "Press {} to play a mono sound (jaguar)",
                    btn_str(Buttons::Action2),
                ));
                draw(format_args!(
                    "Press {} to play a stereo sound (swish)",
                    btn_str(Buttons::Action3),
                ));
                draw(format_args!("Press {} to quit", btn_str(Buttons::Quit),));
                draw(format_args!(""));
                draw(format_args!(
                    "Time {:02}:{:02}:{:02}/{:02}:{:02}:{:02} : {}",
                    ms / 1000 / 60,
                    ms / 1000 % 60,
                    ms / 10 % 100,
                    lenms / 1000 / 60,
                    lenms / 1000 % 60,
                    lenms / 10 % 100,
                    if paused != 0 {
                        "Paused "
                    } else if playing != 0 {
                        "Playing"
                    } else {
                        "Stopped"
                    }
                ));
                draw(format_args!("Channels Playing {channelsplaying}"));
            }
            sleep(50);
        }

        result = FMOD_Sound_Release(sound1);
        errcheck!(result);
        result = FMOD_Sound_Release(sound2);
        errcheck!(result);
        result = FMOD_Sound_Release(sound3);
        errcheck!(result);
        result = FMOD_System_Release(system);
        errcheck!(result);
    }

    close()?;

    Ok(())
}
