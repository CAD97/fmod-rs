use fmod::{raw::*, *};

/// # Plugin support.
impl System {
    /// Specify a base search path for plugins so they can be placed somewhere
    /// else than the directory of the main executable.
    pub fn set_plugin_path(&self, path: &CStr8) -> Result {
        ffi!(FMOD_System_SetPluginPath(self.as_raw(), path.as_ptr() as _))?;
        Ok(())
    }

    // TODO: figure out plugin API
    /*
        /// Loads an FMOD (DSP, Output or Codec) plugin from file.
        ///
        /// Once loaded, DSP plugins can be used via [System::create_dsp_by_plugin],
        /// output plugins can be use via [System::set_output_by_plugin], and codec
        /// plugins will be used automatically.
        ///
        /// When opening a file each codec tests whether it can support the file
        /// format in `priority` order where 0 represents most important and higher
        /// numbers represent less importance.
        ///
        /// The format of the plugin is dependant on the operating system.
        pub fn load_plugin(&self, filename: &CStr8, priority: u32) -> Result<PluginHandle> {
            let mut handle = PluginHandle::default();
            ffi!(FMOD_System_LoadPlugin(
                self.as_raw(),
                filename.as_ptr() as _,
                handle.as_raw_mut(),
                priority,
            ))?;
            Ok(handle)
        }

        /// Unloads an FMOD (DSP, Output or Codec) plugin.
        pub fn unload_plugin(&self, handle: PluginHandle) -> Result {
            ffi!(FMOD_System_UnloadPlugin(self.as_raw(), handle.into_raw()))?;
            Ok(())
        }

        /// Retrieves the number of nested plugins from the selected plugin.
        ///
        /// Most plugins contain a single definition, in which case the count is 1,
        /// however some have a list of definitions. This function returns the
        /// number of plugins that have been defined.
        ///
        /// See the [DSP Plug-in API guide](https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-dsp-plugin-api.html#multiple-plugins-within-one-file)
        /// for more information.
        pub fn get_num_nested_plugins(&self, handle: PluginHandle) -> Result<i32> {
            let mut count = 0;
            ffi!(FMOD_System_GetNumNestedPlugins(
                self.as_raw(),
                handle.into_raw(),
                &mut count,
            ))?;
            Ok(count)
        }

        /// Retrieves the handle of a nested plugin.
        ///
        /// This function is used to iterate handles for plugins that have a list of
        /// definitions.
        ///
        /// Most plugins contain a single definition. If this is the case, only
        /// index 0 is valid, and the returned handle is the same as the handle
        /// passed in.
        pub fn get_nested_plugin(&self, handle: PluginHandle, index: i32) -> Result<PluginHandle> {
            let mut nested_handle = PluginHandle::default();
            ffi!(FMOD_System_GetNestedPlugin(
                self.as_raw(),
                handle.into_raw(),
                index,
                nested_handle.as_raw_mut(),
            ))?;
            Ok(nested_handle)
        }

        /// Retrieves the number of loaded plugins.
        pub fn get_num_plugins(&self, plugin_type: PluginType) -> Result<i32> {
            let mut num_plugins = 0;
            ffi!(FMOD_System_GetNumPlugins(
                self.as_raw(),
                plugin_type.into_raw(),
                &mut num_plugins,
            ))?;
            Ok(num_plugins)
        }

        /// Retrieves the handle of a plugin based on its type and relative index.
        ///
        /// All plugins whether built in or loaded can be enumerated using this and
        /// [System::get_num_plugins].
        pub fn get_plugin_handle(&self, plugin_type: PluginType, index: i32) -> Result<PluginHandle> {
            let mut handle = PluginHandle::default();
            ffi!(FMOD_System_GetPluginHandle(
                self.as_raw(),
                plugin_type.into_raw(),
                index,
                handle.as_raw_mut(),
            ))?;
            Ok(handle)
        }

        // NB: we split get_plugin_info/name into separate calls for two reasons:
        // getting everything *but* the name is cheap, and the name has extra retry
        // requirements to validate non-truncation and UTF-8. This does, however,
        // mean that getting all of the plugin info requires an extra FFI call.

        /// Retrieves information for the selected plugin.
        pub fn get_plugin_info(&self, handle: PluginHandle) -> Result<PluginInfo> {
            let mut kind = PluginType::zeroed();
            let mut version = 0;
            ffi!(FMOD_System_GetPluginInfo(
                self.as_raw(),
                handle.raw,
                kind.as_raw_mut(),
                ptr::null_mut(),
                0,
                &mut version,
            ))?;
            Ok(PluginInfo { kind, version })
        }

        /// Retrieves name for the selected plugin.
        pub fn get_plugin_name(&self, handle: PluginHandle, name: &mut String) -> Result {
            unsafe {
                fmod_get_string(name, |buf| {
                    ffi!(FMOD_System_GetPluginInfo(
                        self.as_raw(),
                        handle.raw,
                        ptr::null_mut(),
                        buf.as_mut_ptr().cast(),
                        buf.len() as _,
                        ptr::null_mut(),
                    ))
                })
            }
        }

        /// Selects an output type given a plugin handle.
        ///
        /// (Windows Only) This function can be called after FMOD is already
        /// initialized. You can use it to change the output mode at runtime. If
        /// [raw::FMOD_SYSTEM_CALLBACK_DEVICELISTCHANGED] is specified use the
        /// set_output call to change to [OutputType::NoSound] if no more sound card
        /// drivers exist.
        pub fn set_output_by_plugin(&self, handle: PluginHandle) -> Result {
            ffi!(FMOD_System_SetOutputByPlugin(
                self.as_raw(),
                handle.into_raw(),
            ))?;
            Ok(())
        }

        /// Retrieves the plugin handle for the currently selected output type.
        pub fn get_output_by_plugin(&self) -> Result<PluginHandle> {
            let mut handle = PluginHandle::default();
            ffi!(FMOD_System_GetOutputByPlugin(
                self.as_raw(),
                handle.as_raw_mut(),
            ))?;
            Ok(handle)
        }

        /// Create a DSP object given a plugin handle.
        ///
        /// A DSP object is a module that can be inserted into the mixing graph to
        /// allow sound filtering or sound generation. See the [DSP architecture guide](https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-dsp-architecture.html)
        /// for more information.
        ///
        /// A handle can come from a newly loaded plugin with [System::load_plugin]
        /// or an existing plugin with [System::get_plugin_handle].
        ///
        /// DSPs must be attached to the DSP graph before they become active, either
        /// via [ChannelControl::add_dsp] or [Dsp::add_input].
        pub fn create_dsp_by_plugin(&self, handle: PluginHandle) -> Result<Handle<'_, Dsp>> {
            let mut dsp = ptr::null_mut();
            ffi!(FMOD_System_CreateDSPByPlugin(
                self.as_raw(),
                handle.into_raw(),
                &mut dsp,
            ))?;
            Ok(unsafe { Handle::new(dsp) })
        }

        // this functionality needs to wait for custom plugin author binding work.
        // pub fn get_dsp_info_by_plugin(&self, handle: PluginHandle) -> Result<DspDescription> {
        // pub fn register_codec(priority: u32, desc: CodecPluginDescription) -> Result<PluginHandle>;
        // pub fn register_dsp(description: DspDescription) -> Result<PluginHandle>;
        // pub fn register_output(description: OutputDescription) -> Result<PluginHandle>;
    */
}

enum_struct! {
    /// Types of plugin used to extend functionality.
    pub enum PluginType: FMOD_PLUGINTYPE {
        /// Audio output interface plugin represented with [OutputDescription].
        Output = FMOD_PLUGINTYPE_OUTPUT,
        /// File format codec plugin represented with [CodecDescription].
        Codec  = FMOD_PLUGINTYPE_CODEC,
        /// DSP unit plugin represented with [DspDescription].
        Dsp    = FMOD_PLUGINTYPE_DSP,
    }
}
