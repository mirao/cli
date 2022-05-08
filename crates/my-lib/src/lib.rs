use std::{fs, path::Path};
pub mod error;

use error::Error;
use log::{debug, trace};
use wapc::WapcHost;
pub struct Module {
    host: WapcHost,
}

impl Module {
    pub fn new(bytes: &[u8]) -> Result<Self, Error> {
        let engine = wasmtime_provider::WasmtimeEngineProvider::new(bytes, None)?;
        let host = WapcHost::new(
            Box::new(engine),
            Some(Box::new(|_id, binding, ns, operation, payload| {
                trace!(
                    "Guest called: binding={}, namespace={}, operation={}, payload={:?}",
                    binding,
                    ns,
                    operation,
                    payload
                );
                Err("Not implemented".into())
            })),
        )?;
        Ok(Module { host })
    }

    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<Self, Error> {
        debug!("Loading wasm file from {:?}", path.as_ref());
        let bytes = fs::read(path.as_ref())
            .map_err(|e| Error::FileNotReadable(path.as_ref().to_path_buf(), e.to_string()))?;
        Self::new(&bytes)
    }

    pub fn run(&self, operation: &str, payload: &[u8]) -> Result<Vec<u8>, Error> {
        debug!("Invoking {}", operation);
        let result = self.host.call(operation, payload)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loads_wasm_file() {
        let result = Module::from_file("./tests/wapc_guest.wasm");
        assert!(result.is_ok());
    }

    #[test]
    fn runs_operation() -> Result<(), Error> {
        let module = Module::from_file("./tests/wapc_guest.wasm")?;

        let bytes = rmp_serde::to_vec("Man").unwrap();
        let payload = module.run("hello", &bytes)?;
        let unpacked: String = rmp_serde::from_slice(&payload).unwrap();
        assert_eq!(unpacked, "Hello, Man.");
        Ok(())
    }
}
