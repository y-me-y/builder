// Copyright (c) 2016-2017 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::error;
use std::fmt;
use std::io;
use std::num;
use std::path::PathBuf;
use std::result;

use actix_web::http::StatusCode;
use actix_web::HttpResponse;

use diesel;
use postgres;
use protobuf;
use r2d2;
use rusoto_s3;
use zmq;

use bldr_core;
use db;
use hab_core;
use hab_net::{self, ErrCode};
use protocol;

#[derive(Debug)]
pub enum Error {
    BadPort(String),
    BuilderCore(bldr_core::Error),
    BusyWorkerUpsert(postgres::error::Error),
    BusyWorkerDelete(postgres::error::Error),
    BusyWorkersGet(postgres::error::Error),
    CaughtPanic(String, String),
    ConnErr(hab_net::conn::ConnErr),
    Db(db::error::Error),
    DbPoolTimeout(r2d2::Error),
    DbTransaction(postgres::error::Error),
    DbTransactionStart(postgres::error::Error),
    DbTransactionCommit(postgres::error::Error),
    DieselError(diesel::result::Error),
    HabitatCore(hab_core::Error),
    InvalidUrl,
    IO(io::Error),
    JobGroupAudit(postgres::error::Error),
    JobGroupCreate(postgres::error::Error),
    JobGroupCancel(postgres::error::Error),
    JobGroupGet(postgres::error::Error),
    JobGroupOriginGet(postgres::error::Error),
    JobGroupPending(postgres::error::Error),
    JobGroupSetState(postgres::error::Error),
    JobGraphPackageInsert(postgres::error::Error),
    JobGraphPackageStats(postgres::error::Error),
    JobGraphPackagesGet(postgres::error::Error),
    JobGroupProjectSetState(postgres::error::Error),
    JobCreate(postgres::error::Error),
    JobGet(postgres::error::Error),
    JobLogArchive(u64, rusoto_s3::PutObjectError),
    JobLogRetrieval(u64, rusoto_s3::GetObjectError),
    JobMarkArchived(postgres::error::Error),
    JobPending(postgres::error::Error),
    JobReset(postgres::error::Error),
    JobSetLogUrl(postgres::error::Error),
    JobSetState(postgres::error::Error),
    SyncJobs(postgres::error::Error),
    LogDirDoesNotExist(PathBuf, io::Error),
    LogDirIsNotDir(PathBuf),
    LogDirNotWritable(PathBuf),
    NetError(hab_net::NetError),
    ParseVCSInstallationId(num::ParseIntError),
    ProjectJobsGet(postgres::error::Error),
    Protobuf(protobuf::ProtobufError),
    Protocol(protocol::ProtocolError),
    UnknownVCS,
    UnknownJobGroup,
    UnknownJobGroupState,
    UnknownJobGraphPackage,
    UnknownJobGroupProjectState,
    UnknownJobState(protocol::ProtocolError),
    Zmq(zmq::Error),
}

pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            Error::BadPort(ref e) => format!("{} is an invalid port. Valid range 1-65535.", e),
            Error::BuilderCore(ref e) => format!("{}", e),
            Error::BusyWorkerUpsert(ref e) => {
                format!("Database error creating or updating a busy worker, {}", e)
            }
            Error::BusyWorkerDelete(ref e) => {
                format!("Database error deleting a busy worker, {}", e)
            }
            Error::BusyWorkersGet(ref e) => {
                format!("Database error retrieving busy workers, {}", e)
            }
            Error::CaughtPanic(ref msg, ref source) => {
                format!("Caught a panic: {}. {}", msg, source)
            }
            Error::ConnErr(ref e) => format!("{}", e),
            Error::Db(ref e) => format!("{}", e),
            Error::DbPoolTimeout(ref e) => {
                format!("Timeout getting connection from the database pool, {}", e)
            }
            Error::DbTransaction(ref e) => format!("Database transaction error, {}", e),
            Error::DbTransactionStart(ref e) => {
                format!("Failed to start database transaction, {}", e)
            }
            Error::DbTransactionCommit(ref e) => {
                format!("Failed to commit database transaction, {}", e)
            }
            Error::DieselError(ref e) => format!("{}", e),
            Error::HabitatCore(ref e) => format!("{}", e),
            Error::InvalidUrl => format!("Bad URL!"),
            Error::IO(ref e) => format!("{}", e),
            Error::JobGroupAudit(ref e) => format!("Database error creating audit entry, {}", e),
            Error::JobGroupCreate(ref e) => format!("Database error creating a new group, {}", e),
            Error::JobGroupCancel(ref e) => format!("Database error canceling a job group, {}", e),
            Error::JobGroupGet(ref e) => format!("Database error getting group data, {}", e),
            Error::JobGroupOriginGet(ref e) => {
                format!("Database error getting group data for an origin, {}", e)
            }
            Error::JobGroupPending(ref e) => format!("Database error getting pending group, {}", e),
            Error::JobGroupSetState(ref e) => format!("Database error setting group state, {}", e),
            Error::JobGraphPackageInsert(ref e) => {
                format!("Database error inserting a new package, {}", e)
            }
            Error::JobGraphPackageStats(ref e) => {
                format!("Database error retrieving package statistics, {}", e)
            }
            Error::JobGraphPackagesGet(ref e) => {
                format!("Database error retrieving packages, {}", e)
            }
            Error::JobGroupProjectSetState(ref e) => {
                format!("Database error setting project state, {}", e)
            }
            Error::JobCreate(ref e) => format!("Database error creating a new job, {}", e),
            Error::JobGet(ref e) => format!("Database error getting job data, {}", e),
            Error::JobLogArchive(job_id, ref e) => {
                format!("Log archiving error for job {}, {}", job_id, e)
            }
            Error::JobLogRetrieval(job_id, ref e) => {
                format!("Log retrieval error for job {}, {}", job_id, e)
            }
            Error::JobMarkArchived(ref e) => {
                format!("Database error marking job as archived, {}", e)
            }
            Error::JobPending(ref e) => format!("Database error getting pending jobs, {}", e),
            Error::JobReset(ref e) => format!("Database error reseting jobs, {}", e),
            Error::JobSetLogUrl(ref e) => format!("Database error setting job log URL, {}", e),
            Error::JobSetState(ref e) => format!("Database error setting job state, {}", e),
            Error::SyncJobs(ref e) => format!("Database error retrieving sync jobs, {}", e),
            Error::LogDirDoesNotExist(ref path, ref e) => {
                format!("Build log directory {:?} doesn't exist!: {:?}", path, e)
            }
            Error::LogDirIsNotDir(ref path) => {
                format!("Build log directory {:?} is not a directory!", path)
            }
            Error::LogDirNotWritable(ref path) => {
                format!("Build log directory {:?} is not writable!", path)
            }
            Error::NetError(ref e) => format!("{}", e),
            Error::ParseVCSInstallationId(ref e) => {
                format!("VCS installation id could not be parsed as u64, {}", e)
            }
            Error::Protobuf(ref e) => format!("{}", e),
            Error::Protocol(ref e) => format!("{}", e),
            Error::ProjectJobsGet(ref e) => {
                format!("Database error getting jobs for project, {}", e)
            }
            Error::UnknownJobGroup => format!("Unknown Group"),
            Error::UnknownJobGroupState => format!("Unknown Group State"),
            Error::UnknownJobGraphPackage => format!("Unknown Package"),
            Error::UnknownJobGroupProjectState => format!("Unknown Project State"),
            Error::UnknownVCS => format!("Unknown VCS"),
            Error::UnknownJobState(ref e) => format!("{}", e),
            Error::Zmq(ref e) => format!("{}", e),
        };
        write!(f, "{}", msg)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BadPort(_) => "Received an invalid port or a number outside of the valid range.",
            Error::BuilderCore(ref err) => err.description(),
            Error::BusyWorkerUpsert(ref err) => err.description(),
            Error::BusyWorkerDelete(ref err) => err.description(),
            Error::BusyWorkersGet(ref err) => err.description(),
            Error::CaughtPanic(_, _) => "Caught a panic",
            Error::ConnErr(ref err) => err.description(),
            Error::Db(ref err) => err.description(),
            Error::DbPoolTimeout(ref err) => err.description(),
            Error::DbTransaction(ref err) => err.description(),
            Error::DbTransactionCommit(ref err) => err.description(),
            Error::DbTransactionStart(ref err) => err.description(),
            Error::DieselError(ref err) => err.description(),
            Error::HabitatCore(ref err) => err.description(),
            Error::IO(ref err) => err.description(),
            Error::InvalidUrl => "Bad Url!",
            Error::JobGroupAudit(ref err) => err.description(),
            Error::JobGroupCreate(ref err) => err.description(),
            Error::JobGroupCancel(ref err) => err.description(),
            Error::JobGroupGet(ref err) => err.description(),
            Error::JobGroupOriginGet(ref err) => err.description(),
            Error::JobGroupPending(ref err) => err.description(),
            Error::JobGroupSetState(ref err) => err.description(),
            Error::JobGraphPackageInsert(ref err) => err.description(),
            Error::JobGraphPackageStats(ref err) => err.description(),
            Error::JobGraphPackagesGet(ref err) => err.description(),
            Error::JobGroupProjectSetState(ref err) => err.description(),
            Error::JobCreate(ref err) => err.description(),
            Error::JobGet(ref err) => err.description(),
            Error::JobLogArchive(_, ref err) => err.description(),
            Error::JobLogRetrieval(_, ref err) => err.description(),
            Error::JobMarkArchived(ref err) => err.description(),
            Error::JobPending(ref err) => err.description(),
            Error::JobReset(ref err) => err.description(),
            Error::JobSetLogUrl(ref err) => err.description(),
            Error::JobSetState(ref err) => err.description(),
            Error::SyncJobs(ref err) => err.description(),
            Error::LogDirDoesNotExist(_, ref err) => err.description(),
            Error::LogDirIsNotDir(_) => "Build log directory is not a directory",
            Error::LogDirNotWritable(_) => "Build log directory is not writable",
            Error::NetError(ref err) => err.description(),
            Error::ParseVCSInstallationId(_) => "VCS installation id could not be parsed as u64",
            Error::ProjectJobsGet(ref err) => err.description(),
            Error::Protobuf(ref err) => err.description(),
            Error::Protocol(ref err) => err.description(),
            Error::UnknownJobState(ref err) => err.description(),
            Error::UnknownJobGroup => "Unknown Group",
            Error::UnknownJobGroupState => "Unknown Group State",
            Error::UnknownJobGraphPackage => "Unknown Package",
            Error::UnknownJobGroupProjectState => "Unknown Project State",
            Error::UnknownVCS => "Unknown VCS",
            Error::Zmq(ref err) => err.description(),
        }
    }
}

impl Into<HttpResponse> for Error {
    fn into(self) -> HttpResponse {
        match self {
            Error::NetError(ref e) => HttpResponse::build(net_err_to_http(&e)).json(&e),
            Error::BuilderCore(ref e) => HttpResponse::new(bldr_core_err_to_http(e)),
            Error::DieselError(ref e) => HttpResponse::new(diesel_err_to_http(e)),

            // Default
            _ => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

fn bldr_core_err_to_http(err: &bldr_core::Error) -> StatusCode {
    match err {
        bldr_core::error::Error::RpcError(code, _) => StatusCode::from_u16(*code).unwrap(),
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn diesel_err_to_http(err: &diesel::result::Error) -> StatusCode {
    match err {
        diesel::result::Error::NotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn net_err_to_http(err: &hab_net::NetError) -> StatusCode {
    match err.code() {
        ErrCode::TIMEOUT => StatusCode::GATEWAY_TIMEOUT,
        ErrCode::REMOTE_REJECTED => StatusCode::NOT_ACCEPTABLE,
        ErrCode::ENTITY_NOT_FOUND => StatusCode::NOT_FOUND,
        ErrCode::ENTITY_CONFLICT => StatusCode::CONFLICT,

        ErrCode::ACCESS_DENIED | ErrCode::SESSION_EXPIRED => StatusCode::UNAUTHORIZED,

        ErrCode::BAD_REMOTE_REPLY | ErrCode::SECRET_KEY_FETCH | ErrCode::VCS_CLONE => {
            StatusCode::BAD_GATEWAY
        }

        ErrCode::NO_SHARD | ErrCode::SOCK | ErrCode::REMOTE_UNAVAILABLE => {
            StatusCode::SERVICE_UNAVAILABLE
        }

        ErrCode::BAD_TOKEN => StatusCode::FORBIDDEN,

        ErrCode::GROUP_NOT_COMPLETE
        | ErrCode::BUILD
        | ErrCode::EXPORT
        | ErrCode::POST_PROCESSOR
        | ErrCode::SECRET_KEY_IMPORT
        | ErrCode::INVALID_INTEGRATIONS => StatusCode::UNPROCESSABLE_ENTITY,

        ErrCode::PARTIAL_JOB_GROUP_PROMOTE => StatusCode::PARTIAL_CONTENT,

        ErrCode::BUG
        | ErrCode::SYS
        | ErrCode::DATA_STORE
        | ErrCode::WORKSPACE_SETUP
        | ErrCode::REG_CONFLICT
        | ErrCode::REG_NOT_FOUND => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl From<bldr_core::Error> for Error {
    fn from(err: bldr_core::Error) -> Error {
        Error::BuilderCore(err)
    }
}

impl From<hab_core::Error> for Error {
    fn from(err: hab_core::Error) -> Error {
        Error::HabitatCore(err)
    }
}

impl From<db::error::Error> for Error {
    fn from(err: db::error::Error) -> Self {
        Error::Db(err)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Error {
        Error::DieselError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IO(err)
    }
}

impl From<hab_net::NetError> for Error {
    fn from(err: hab_net::NetError) -> Self {
        Error::NetError(err)
    }
}

impl From<hab_net::conn::ConnErr> for Error {
    fn from(err: hab_net::conn::ConnErr) -> Self {
        Error::ConnErr(err)
    }
}

impl From<protobuf::ProtobufError> for Error {
    fn from(err: protobuf::ProtobufError) -> Error {
        Error::Protobuf(err)
    }
}

impl From<protocol::ProtocolError> for Error {
    fn from(err: protocol::ProtocolError) -> Self {
        Error::Protocol(err)
    }
}

impl From<zmq::Error> for Error {
    fn from(err: zmq::Error) -> Error {
        Error::Zmq(err)
    }
}
