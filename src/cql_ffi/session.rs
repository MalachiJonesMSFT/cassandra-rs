#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(missing_copy_implementations)]

use std::ffi::CString;

use cql_ffi::batch::CassBatch;
use cql_ffi::future::CassFuture;
use cql_ffi::future::ResultFuture;
use cql_ffi::future::PreparedFuture;
use cql_ffi::error::CassError;
use cql_ffi::statement::CassStatement;
use cql_ffi::schema::CassSchema;
use cql_ffi::cluster::Cluster;
use cql_bindgen::CassFuture as _CassFuture;
use cql_bindgen::cass_future_free;
use cql_bindgen::cass_future_wait;
use cql_bindgen::cass_future_error_code;

use cql_bindgen::CassSession as _Session;
use cql_bindgen::cass_session_new;
use cql_bindgen::cass_session_free;
use cql_bindgen::cass_session_close;
use cql_bindgen::cass_session_connect;
use cql_bindgen::cass_session_prepare;
use cql_bindgen::cass_session_execute;
use cql_bindgen::cass_session_execute_batch;
use cql_bindgen::cass_session_get_schema;
use cql_bindgen::cass_session_connect_keyspace;

pub struct Session(pub *mut _Session);

unsafe impl Sync for Session{}
unsafe impl Send for Session{}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            cass_session_free(self.0)
        }
    }
}

impl Session {
    /// Create a new Cassanda session.
    /// It's recommended to use Cluster.connect() instead
    pub fn new() -> Session {
        unsafe {
            Session(cass_session_new())
        }
    }

    pub fn close(self) -> CassFuture {
        unsafe {
            CassFuture(cass_session_close(self.0))
        }
    }

    pub fn connect(self, cluster: &Cluster) -> SessionFuture {
        unsafe {
            SessionFuture(cass_session_connect(self.0, cluster.0), self)
        }
    }

    pub fn prepare(&self, query: &str) -> Result<PreparedFuture, CassError> {
        unsafe {
            let query = CString::new(query).unwrap();
            Ok(PreparedFuture(cass_session_prepare(self.0, query.as_ptr())))
        }
    }

    pub fn execute(&self, statement: &str, parameter_count: u64) -> ResultFuture {
        unsafe {
            ResultFuture(cass_session_execute(self.0,
                                              CassStatement::new(statement,parameter_count).0))
        }
    }

    pub fn execute_statement(&self, statement: &CassStatement) -> ResultFuture {
        unsafe {
            ResultFuture(cass_session_execute(self.0, statement.0))
        }
    }

    pub fn execute_batch(&self, batch: CassBatch) -> ResultFuture {
        ResultFuture(unsafe {
                cass_session_execute_batch(self.0, batch.0)
            })
    }

    pub fn get_schema(&self) -> CassSchema {
        unsafe {
            CassSchema(cass_session_get_schema(self.0))
        }
    }

    pub unsafe fn connect_keyspace(&self,
                                   cluster: Cluster,
                                   keyspace: *const ::libc::c_char)
                                   -> CassFuture {
        CassFuture(cass_session_connect_keyspace(self.0, cluster.0, keyspace))
    }
}

pub struct SessionFuture(pub *mut _CassFuture, pub Session);

impl SessionFuture {
    pub fn wait(self) -> Result<Session, CassError> {
        unsafe {
            cass_future_wait(self.0);
            self.error_code()
        }
    }

    fn error_code(self) -> Result<Session, CassError> {
        unsafe {
            let code = cass_future_error_code(self.0);
            cass_future_free(self.0);
            CassError::build(code).wrap(self.1)
        }
    }
}
