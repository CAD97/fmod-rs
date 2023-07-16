use {
    crate::utils::fmod_get_string,
    fmod::{raw::*, *},
    std::time::Duration,
};

/// # Network configuration.
impl System {
    /// Set a proxy server to use for all subsequent internet connections.
    ///
    /// Specify the proxy in `host:port` format e.g. `www.fmod.com:8888`
    /// (defaults to port 80 if no port is specified).
    ///
    /// Basic authentication is supported using `user:password@host:port` format
    /// e.g. `bob:sekrit123@www.fmod.com:8888`.
    pub fn set_network_proxy(&self, proxy: &CStr8) -> Result {
        ffi!(FMOD_System_SetNetworkProxy(
            self.as_raw(),
            proxy.as_ptr() as _,
        ))?;
        Ok(())
    }

    /// Retrieves the URL of the proxy server used in internet streaming.
    pub fn get_network_proxy(&self, proxy: &mut String) -> Result {
        unsafe {
            fmod_get_string(proxy, |buf| {
                ffi!(FMOD_System_GetNetworkProxy(
                    self.as_raw(),
                    buf.as_mut_ptr().cast(),
                    buf.len() as _,
                ))
            })
        }
    }

    /// Set the timeout for network streams.
    pub fn set_network_timeout(&self, timeout: Duration) -> Result {
        let timeout = timeout.as_millis() as _;
        ffi!(FMOD_System_SetNetworkTimeout(self.as_raw(), timeout))?;
        Ok(())
    }

    /// Retrieve the timeout value for network streams.
    pub fn get_network_timeout(&self) -> Result<Duration> {
        let mut timeout = 0;
        ffi!(FMOD_System_GetNetworkTimeout(self.as_raw(), &mut timeout))?;
        Ok(Duration::from_millis(timeout as _))
    }
}
