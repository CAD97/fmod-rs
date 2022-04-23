# Studio API

(This document is from the FMOD API User Manual 2.02.06 (2022-03-14). © 2022 Firelight Technologies Pty Ltd.)

 ## What is the Studio API?

 The FMOD Studio API allows programmers to interact with the data driven projects created via FMOD Studio at run time. It is built on top of the Core API and provides additional functionality to what the Core API provides.

 Studio API wrappers are available for C, C++ and C# as fmod_studio.h, fmod_studio.hpp and fmod_studio.cs respectively. Any includes and libraries required by the Core API are also required for the Studio API.

 ## Getting Started

 At the most basic level, this is achieved by creating the [`studio::System`](studio::System "") object and calling [`studio::System::initialize`](studio::System::initialize "Initializes the Studio System.") on it. You need to do this once when your game starts before you can use FMOD, once the system has been initialized you can start loading banks and creating event instances without having to do any other preparations. A more detailed description of initialization can be found in the FMOD [Getting Started](<https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-getting-started.html>) white paper.

 If using the C# wrapper you will need to call a Core API function before calling anything else in the Studio API. Some runtimes do not perform dependency loading, and because the Studio API depends on the Core API, fmod.dll needs to be loaded before fmod_studio.dll - this can be ensured by calling a Core API function before [`studio::System::create`](studio::System::create "FMOD Studio System creation function.")

 ## Creating the Studio System

 Instances of [`studio::System`](studio::System "") must be created by calling [`studio::System::create`](studio::System::create "FMOD Studio System creation function."). Once created an instance must be initialized with a call to [`studio::System::initialize`](studio::System::initialize "Initializes the Studio System.") before it can be used. [`studio::System::create`](studio::System::create "FMOD Studio System creation function.") also creates an FMOD [`System`](System "") instance which can be retrieved using [`studio::System::getCoreSystem`](studio::System::getCoreSystem "Retrieves the Core System.").

 Pre-initialization configuration of both the Studio System and the Core System may be performed prior to calling [`studio::System::initialize`](studio::System::initialize "Initializes the Studio System."):

  - [`studio::System::set_advanced_settings`](studio::System::set_advanced_settings "Sets advanced settings.") can be called to configure various advanced settings.
 - [`System::set_software_format`](System::set_software_format "Sets the output format for the software mixer.") should be called on the Core System object with speakermode corresponding to the project's output format if there is a possibility of the output audio device not matching the project's format. Any differences between the project format and the Core System's speakermode will cause the mix to be incorrect.
 
 The Studio System is shut down and released by calling [`studio::System::release`](studio::System::release "Shut down and free the Studio System object."), make sure to call this before your game is exited.

 ## Studio System Processing

 FMOD Studio is built on a multithreaded processing model where API calls on a game thread try to be fast by only reading shadow data, writing shadow data, or enqueuing commands to a buffer, while a separate Studio update thread triggered by the mixer asynchronously processes the API commands and performs all the logic required by event playback and automation. 

 When running in this default asynchronous processing mode, calling [`studio::System::update`](studio::System::update "Update the FMOD Studio System.") from your game is a fast operation which submits the queued command buffer to the asynchronous thread and performs any asynchronous callbacks due to processing on the Studio update thread.

 Studio may also be initialized in synchronous mode with the [`FMOD_STUDIO_INIT_SYNCHRONOUS_UPDATE`](FMOD_STUDIO_INIT_SYNCHRONOUS_UPDATE "") flag. When operating in synchronous mode, API calls behave the same but all the processing of queued commands and event playback and automation is performed when your game calls [`studio::System::update`](studio::System::update "Update the FMOD Studio System.").

 If you do not call [`studio::System::update`](studio::System::update "Update the FMOD Studio System.") then previous commands will not be executed. While most of the API hides this behavior with use of shadowed variables, it can cause unexpected results if waiting in a loop for [`studio::EventDescription::getSampleLoadingState`](studio::EventDescription::getSampleLoadingState "Retrieves the sample data loading state.") or [`studio::Bank::getLoadingState`](studio::Bank::getLoadingState "Retrieves the loading state.") without calling update first.

 ## Bank Layout

 The FMOD Studio Bank file contains event metadata and sound data in the one file.

 ![Studio Bank Layout](https://fmod.com/docs/2.02/api/images/studio-bank-layout.png)

 Loading a bank will load all metadata, which contains information about all the events, parameters, and other data needed for all events associated with that bank.

 The sound sample data has two different types, normal sample data, and streaming sound data. Normal sample data can be loaded per event type. Streaming data is streamed in on demand as events are played, and is never fully loaded ahead of time. Streaming data is good for music, voice over, and other sounds which are of a long duration. Whether sound data is streamed or not is set up by the designer in FMOD Studio and cannot be changed at runtime.

 ## Bank Loading

 Banks are loaded by calling [`studio::System::loadBankFile`](studio::System::loadBankFile "Loads the metadata of a Studio bank from file."). They are unloaded by [`studio::Bank::unload`](studio::Bank::unload "Unloads the bank.").

 Bank loading can be controlled with the [`FMOD_STUDIO_LOAD_BANK_FLAGS`](FMOD_STUDIO_LOAD_BANK_FLAGS "Flags to controlling bank loading."). When loading banks with [`FMOD_STUDIO_LOAD_BANK_NORMAL`](FMOD_STUDIO_LOAD_BANK_NORMAL ""), the function will not return until the bank has completed loading. When using the [`FMOD_STUDIO_LOAD_BANK_NONBLOCKING`](FMOD_STUDIO_LOAD_BANK_NONBLOCKING "") flag, the load bank function will return before the bank has completed loading.

 As soon as a bank has completed loaded, all the metadata in it can be accessed. This means that event descriptions can be found with [`studio::System::getEvent`](studio::System::getEvent "Retrieves an EventDescription."), and instances created from those descriptions. The bank loading state can be queried with [`studio::Bank::getLoadingState`](studio::Bank::getLoadingState "Retrieves the loading state.").

 ### Strings Bank

 The Strings bank is a special bank which contains the string lookup of event path to GUID. The strings bank functions identically to a normal bank except that it never contains sample or streaming sound data.

 ## Bank Unload

 Banks can be unloaded by calling [`studio::Bank::unload`](studio::Bank::unload "Unloads the bank."). Unloading a bank will free all sample data, invalidate the events descriptions belonging to that bank, and destroy associated instances.

 If the bank containing the sample data was loaded via [`studio::System::loadBankMemory`](studio::System::loadBankMemory "Loads the metadata of a Studio bank from memory.") then the system must immediately unload sample data when the bank is unloaded. This may lead to playback errors if any instances of events in the bank are still playing. This can occur even if multiple copies of the sample data are loaded from different banks and only one of the copies is being unloaded.

 ## Sample Data loading

 Sample data is loaded from one of the three actions:

  - [`studio::Bank::loadSampleData`](studio::Bank::loadSampleData "Loads non-streaming sample data for all events in the bank.")
 - [`studio::EventDescription::loadSampleData`](studio::EventDescription::loadSampleData "Loads non-streaming sample data used by the event.")
 - [`studio::EventDescription::createInstance`](studio::EventDescription::createInstance "Creates a playable instance.")
 
 For cases where most or all of the events may play at any time, then loading calling [`studio::Bank::loadSampleData`](studio::Bank::loadSampleData "Loads non-streaming sample data for all events in the bank.") to load all data up front may be the best approach. Once the bank sample data has loaded, then all event instances can be created or destroyed and use that existing data immediately. However, it does have the highest memory overhead. Repeated calls to [`studio::Bank::loadSampleData`](studio::Bank::loadSampleData "Loads non-streaming sample data for all events in the bank.") are reference counted, and the bank's sample data is only unloaded when [`studio::Bank::unloadSampleData`](studio::Bank::unloadSampleData "Unloads non-streaming sample data for all events in the bank.") has been called an equal number of times.

 Sample data can be loaded for a selected event using [`studio::EventDescription::loadSampleData`](studio::EventDescription::loadSampleData "Loads non-streaming sample data used by the event."). It is best to load the sample data ahead of time, so that the event's sound sample data is ready when needed. For cases of very common events, the sample data could be loaded for the duration of the game or level. For less common events, the sample data may be loaded in or out as needed. Repeated calls to [`studio::EventDescription::loadSampleData`](studio::EventDescription::loadSampleData "Loads non-streaming sample data used by the event.") are reference counted, and the bank's sample data is only unloaded when [`studio::EventDescription::unloadSampleData`](studio::EventDescription::unloadSampleData "Unloads all non-streaming sample data.") has been called an equal number of times, or if the entire bank is unloaded.

 When either of these reference counts is incremented to one the system begins loading the referenced sample data. The sample data is loaded asynchronously and the loading state may be polled by calling [`studio::Bank::getSampleLoadingState`](studio::Bank::getSampleLoadingState "Retrieves the loading state of the samples in the bank.") or [`studio::EventDescription::getSampleLoadingState`](studio::EventDescription::getSampleLoadingState "Retrieves the sample data loading state."). Alternatively, you can call [`studio::System::flushSampleLoading`](studio::System::flushSampleLoading "Block until all sample loading and unloading has completed."), which will block until all sample loading and unloading has completed.

 When an instance of an event is created by calling [`studio::EventDescription::createInstance`](studio::EventDescription::createInstance "Creates a playable instance.") the system begins loading any non-streaming sample data which is not already loaded or loading. Once again the sample data is loaded asynchronously and the loading state may be polled by calling [`studio::EventDescription::getSampleLoadingState`](studio::EventDescription::getSampleLoadingState "Retrieves the sample data loading state."). If playback of the instance is started by calling [`studio::EventInstance::start`](studio::EventInstance::start "Starts playback.") before the sample data has finished loading then the instance will stay in the [`FMOD_STUDIO_PLAYBACK_STARTING`](FMOD_STUDIO_PLAYBACK_STARTING "") state until loading of the sampled data has completed.

 The automatic loading of sample data is the simplest approach and uses the least amount of memory. However it has the following disadvantages:

  - Sample data will only start loading when the instance is created, which may be just before [`studio::EventInstance::start`](studio::EventInstance::start "Starts playback.") is called.
 - Sample data will only stay loaded for as long as at least one instance exists.
 
 For the case of one-shots, this may mean that the sample data is constantly loaded and unloaded whenever a one-shot plays, which is not a good approach. For these sort of common sounds, it is better to call [`studio::EventDescription::loadSampleData`](studio::EventDescription::loadSampleData "Loads non-streaming sample data used by the event.") so the sample data stays in memory rather than constantly unloading then reloading it.

 The three approaches to bank loading can be combined. The sample data will stay loaded for as long as at least one of the three conditions are met.

 ### Idle Pool

 For users who don't explicitly load sample data, sounds will be loaded and unloaded on demand. To help avoid unnecessary file access, there is an idle pool for recently used sounds. When a sound is no longer needed (e.g due to an event instance finishing), its sample data will be placed into the idle pool to be reused later if needed.

 By default, the idle pool is set to 256kB in size. This can be customized via the [`FMOD_STUDIO_ADVANCEDSETTINGS::idleSampleDataPoolSize`](FMOD_STUDIO_ADVANCEDSETTINGS::idleSampleDataPoolSize "") field.

 ## Dialogue and Localization

 Start by [loading the banks](<https://fmod.com/resources/documentation-api?version=2.02&page=#bank-loading>) that contain the audio tables. Next, create an instance of an event that contains a programmer instrument. Using this instance, you will be able to register for [event callbacks](<https://fmod.com/resources/documentation-api?version=2.02&page=#event-callbacks>), specifically [`FMOD_STUDIO_EVENT_CALLBACK_CREATE_PROGRAMMER_SOUND`](FMOD_STUDIO_EVENT_CALLBACK_CREATE_PROGRAMMER_SOUND ""). Using the callbacks you will be able to create and assign sounds from the asset tables.

 For localized dialogue, make sure that the required localized bank has been loaded. Ensure that any other localizations on the same bank are unloaded before loading a new localization.

 See the [FMOD Studio Documentation](<https://fmod.com/resources/documentation-api?version=2.02&page=https://fmod.com/resources/documentation-studio?version=2.2&page=dialogue-and-localization.html>) for more information. 

 ### Scripting Example

 This is a modified excerpt of the programmer instrument example that is included in the C++ FMOD Studio API installation folder. The error checking has been removed for brevity.

``````````cpp
struct ProgrammerSoundContext
{
    FMOD::System* coreSystem;
    FMOD::studio::System* system;
    const char* dialogueString;
};

ProgrammerSoundContext programmerSoundContext;
programmerSoundContext.system = system;
programmerSoundContext.coreSystem = coreSystem;
``````````

 This section is to set up a struct to contain the various systems required for injecting audio files or loading keys into a programmer instrument.

``````````cpp
eventInstance->setUserData(&programmerSoundContext);
eventInstance->setCallback(programmerSoundCallback, FMOD_STUDIO_EVENT_CALLBACK_CREATE_PROGRAMMER_SOUND | FMOD_STUDIO_EVENT_CALLBACK_DESTROY_PROGRAMMER_SOUND);
``````````

 The `setUserData()` function allows you to attach any kind of data to the event instance. In this case, the Studio system, Core system, and the dialogue string are being attached to this event instance.

 The `setCallback()` function attaches the callback to the event instance. This callback will be set up outside the main thread and explained more later on.

``````````cpp
// Available banks
// "Dialogue_EN.bank", "Dialogue_JP.bank", "Dialogue_CN.bank"
FMOD::studio::Bank* localizedBank = NULL;
system->loadBankFile(Common_MediaPath("Dialogue_JP.bank"), FMOD_STUDIO_LOAD_BANK_NORMAL, &localizedBank);
programmerSoundContext.dialogueString = "welcome";
eventInstance->start();
``````````

 The way to set up different localizations is to separate the languages into different banks with their own audio tables. The `programmerSoundContext.dialogueString` variable is the audio table key you wish to use. In this example, "welcome" is used.

 With "welcome" as the key, what sound plays depends on the bank loaded. In this example, it will play the Japanese bank's "welcome" audio file.

``````````cpp
FMOD_RESULT F_CALLBACK programmerSoundCallback(FMOD_STUDIO_EVENT_CALLBACK_TYPE type, FMOD_STUDIO_EVENTINSTANCE* event, void* parameters)
``````````

 This function is to set up what happens when a programmer instrument callback is called.

``````````cpp
{
    FMOD::studio::EventInstance* eventInstance = (FMOD::studio::EventInstance*)event;

    if (type == FMOD_STUDIO_EVENT_CALLBACK_CREATE_PROGRAMMER_SOUND)
    {
        // Get our context from the event instance user data
        ProgrammerSoundContext* context = NULL;
        eventInstance->getUserData((void**)&context);

        // Find the audio file in the audio table with the key
        FMOD_STUDIO_SOUND_INFO info;
        context->system->getSoundInfo(context->dialogueString, &info);

        FMOD::Sound* sound = NULL;
        context->coreSystem->create_sound(info.name_or_data, FMOD_LOOP_NORMAL | Mode::CreateCompressedSample | Mode::NonBlocking | info.mode, &info.exinfo, &sound);

        FMOD_STUDIO_PROGRAMMER_SOUND_PROPERTIES* props = (FMOD_STUDIO_PROGRAMMER_SOUND_PROPERTIES*)parameters;

        // Pass the sound to FMOD
        props->sound = (FMOD_SOUND*)sound;
        props->subsoundIndex = info.subsoundindex;
    }
``````````

 The `context` struct set up previously is attached to the event with `getUserData()`. As mentioned previously, these are the Core system, the Studio system, and the dialogue string.

 The `context` struct's dialogue string is passed to the `context` struct's Studio system. The system will search all loaded audio tables for the string provided. The sound info is then passed into the `info` variable. If multiple audio tables are loaded that contain the same key, the latest loaded audio table is used.

 A Core level FMOD::Sound is then instantiated using the information gathered in the `info` variable. The audio table is passed in as `info.name_or_data` but the specific audio file to be used will be specified later.

 In the event of a programmer instrument being created (triggered) the programmer instrument expects a `FMOD_STUDIO_PROGRAMMER_SOUND_PROPERTIES` to be passed into it. The audio table, in the FMOD::Sound, is provided to the `props` properties and the `subsoundIndex` is the actual audio file (subsound of the audio table) chosen with the key string.

``````````cpp
    else if (type == FMOD_STUDIO_EVENT_CALLBACK_DESTROY_PROGRAMMER_SOUND)
    {
        FMOD_STUDIO_PROGRAMMER_SOUND_PROPERTIES* props = (FMOD_STUDIO_PROGRAMMER_SOUND_PROPERTIES*)parameters;

        // Obtain the sound
        FMOD::Sound* sound = (FMOD::Sound*)props->sound;

        // Release the sound
        sound->release();
    }
}
``````````

 When the programmer instrument has been untriggered, either by the instrument no longer meeting its conditions in the event or by the event stopping, then it will fire the `FMOD_STUDIO_EVENT_CALLBACK_DESTROY_PROGRAMMER_SOUND` callback. In the above code, when the programmer instrument is destroyed, it will find the FMOD::Sound passed into it and release it, freeing the memory.

 ## Playing Events

 An event is an instanceable unit of sound content that can be triggered, controlled and stopped from game code. Everything that produces a sound in a game should have a corresponding event.

 An event contains and is composed of tracks, instruments, and parameters. The parameters trigger the instruments, which route audio content into the tracks. The tracks route into other tracks, or into the event's master track; The output of the event's master track routes into the project mixer. In addition, the event's parameters can control and manipulate most properties of the event, of the event's instruments and logic markers, and of effect modules on the event's tracks.

 In studio api, typically you will load a [`studio::EventDescription`](studio::EventDescription "") from a [`studio::Bank`](studio::Bank ""). Then you will use the [`studio::EventDescription`](studio::EventDescription "") to create an [`studio::EventInstance`](studio::EventInstance ""). Then to play the event, you call [`studio::EventInstance::start`](studio::EventInstance::start "Starts playback."). Generally it is a best practice to call [`studio::EventInstance::release`](studio::EventInstance::release "Marks the event instance for release.") on event instances immediately after starting them, unless you want to play the event instance multiple times or explicitly stop it and start it again later.

 ## Event Callbacks

 FMOD Studio allows the user to specify a callback function to call when various state changes occur in an event instance. See [`FMOD_STUDIO_EVENT_CALLBACK_TYPE`](FMOD_STUDIO_EVENT_CALLBACK_TYPE "Studio event callback types.") for the full list of callbacks available. The callback can be set automatically for all new instances of an event using [`studio::EventDescription::setCallback`](studio::EventDescription::setCallback "Sets the user callback."), or it can be set manually for individual event instances using [`studio::EventInstance::setCallback`](studio::EventInstance::setCallback "Sets the user callback.").

 Some callbacks may be fired asynchronously on a thread owned by FMOD, depending on Studio initialization flags.

 When Studio has been initialized in asynchronous mode (the default mode), callbacks are fired from the Studio asynchronous thread as they occur.

 If Studio has been initialized with [`FMOD_STUDIO_INIT_DEFERRED_CALLBACKS`](FMOD_STUDIO_INIT_DEFERRED_CALLBACKS "") then the [`FMOD_STUDIO_EVENT_CALLBACK_TIMELINE_MARKER`](FMOD_STUDIO_EVENT_CALLBACK_TIMELINE_MARKER "") and  
 [`FMOD_STUDIO_EVENT_CALLBACK_TIMELINE_BEAT`](FMOD_STUDIO_EVENT_CALLBACK_TIMELINE_BEAT "") callbacks will be fired from the next call to [`studio::System::update`](studio::System::update "Update the FMOD Studio System.").

 If Studio has been initialized with [`FMOD_STUDIO_INIT_SYNCHRONOUS_UPDATE`](FMOD_STUDIO_INIT_SYNCHRONOUS_UPDATE "") then all callbacks will be fired from the next call to [`studio::System::update`](studio::System::update "Update the FMOD Studio System.").

 **See Also:** [Callback Behavior](<https://fmod.com/resources/documentation-api?version=2.02&page=glossary.html#callback-behavior>)

 ## Setting Parameters

 Parameters are used in FMOD Studio to control the behavior of events.

 They can exist on an event:

  - [`studio::EventInstance::setParameterByName`](studio::EventInstance::setParameterByName "Sets a parameter value by name.")
 - [`studio::EventInstance::setParameterByID`](studio::EventInstance::setParameterByID "Sets a parameter value by unique identifier.")
 
 or a global basis:

  - [`studio::System::setParameterByName`](studio::System::setParameterByName "Sets a global parameter value by name.")
 - [`studio::System::setParameterByID`](studio::System::setParameterByID "Sets a global parameter value by unique identifier.")
 
 ## Spatialization (3D)

 Audio spatialization is the process of taking an audio file and making it sound "in the world".  
 See the [Studio API 3D Events](<https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-studio-3d-events.html>) and [Spatial Audio](<https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-spatial-audio.html>) white papers.

 ## Working with Reverb

 Reverb in the FMOD Studio API can be handled in two ways. The sound designer can add reverb effects to the master bus or individual events, and control the levels sent to those effects using Studio. The second approach is to use the core reverb system. 

 The core system has four user configurable 3d reverbs. Studio event instances can interact with the core reverb system by sending their signal to the core reverbs. The send level can be set with [`studio::EventInstance::setReverbLevel`](studio::EventInstance::setReverbLevel "Sets the core reverb send level.") and queried with [`studio::EventInstance::getReverbLevel`](studio::EventInstance::getReverbLevel "Retrieves the core reverb send level.").

 ## Signal Paths

 Each event or bus has a signal path to the master bus. The signal path is composed of all buses that receive signal from the event or bus. This includes any buses on the direct path to the master bus as well as any buses that are targeted by sends.

 By default, when an event instance is created, the system ensures that every bus on its signal path has a corresponding channel group. When an event instance is destroyed, the system destroys any channel groups which are no longer required.

 You can override the default behavior by calling [`studio::Bus::lockChannelGroup`](studio::Bus::lockChannelGroup "Locks the core channel group."). This forces the system to ensure the channel group exists for that bus and each bus on its signal path. The system cannot destroy any of these channel groups until you call [`studio::Bus::unlockChannelGroup`](studio::Bus::unlockChannelGroup "Unlocks the core channel group.").
