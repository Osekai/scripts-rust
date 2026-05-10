use std::{error::Error, net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use eyre::{Context as _, Result};
use rosu_v2::{error::OsuError, model::GameMode, prelude::UserExtended, Osu, OsuResult};
use tokio::signal;

use crate::{
    database::Database,
    model::{OsuUser, UserFull},
};

#[derive(Clone)]
pub struct ServerState {
    osu: Arc<Osu>,
    mysql: Database,
}

impl ServerState {
    pub fn new(osu: Arc<Osu>, mysql: Database) -> Self {
        Self { osu, mysql }
    }

    async fn add_user_handler(
        State(state): State<ServerState>,
        Path(user_id): Path<u32>,
    ) -> StatusCode {
        info!("Received /add/{user_id} request");

        let osu = &state.osu;

        let (std_res, tko_res, ctb_res, mna_res) = tokio::join!(
            osu.user(user_id).mode(GameMode::Osu),
            osu.user(user_id).mode(GameMode::Taiko),
            osu.user(user_id).mode(GameMode::Catch),
            osu.user(user_id).mode(GameMode::Mania),
        );

        let user = match gather_user(osu, user_id, std_res, tko_res, ctb_res, mna_res).await {
            Ok(user) => user,
            Err(err) => {
                error!(user_id, ?err, "Failed to request user from osu! API");

                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };

        match &user {
            OsuUser::Available(UserFull {
                user_id, username, ..
            }) => info!(user_id, username, "Successfully retrieved user"),
            OsuUser::Restricted { user_id: id } => warn!("User {id} is restricted"),
        }

        let users = std::slice::from_ref(&user);
        let user_medals = state.mysql.store_user_medals(users);
        let usernames = state.mysql.update_usernames(users);
        tokio::join!(user_medals, usernames);

        StatusCode::OK
    }
}

async fn gather_user(
    osu: &Arc<Osu>,
    user_id: u32,
    std_res: OsuResult<UserExtended>,
    tko_res: OsuResult<UserExtended>,
    ctb_res: OsuResult<UserExtended>,
    mna_res: OsuResult<UserExtended>,
) -> OsuResult<OsuUser> {
    async fn handle_res(
        osu: &Arc<Osu>,
        res: OsuResult<UserExtended>,
        mode: GameMode,
        user_id: u32,
    ) -> OsuResult<Option<UserExtended>> {
        match res {
            Ok(user) => Ok(Some(user)),
            Err(OsuError::NotFound) => Ok(None),
            Err(OsuError::Request { source })
                if source
                    .source()
                    .is_some_and(|err| err.to_string().starts_with("http2 error")) =>
            {
                osu.user(user_id).mode(mode).await.map(Some)
            }
            Err(err) => Err(err),
        }
    }

    macro_rules! get_or_restricted {
        ( $res:ident, $mode:ident ) => {
            match handle_res(osu, $res, GameMode::$mode, user_id).await? {
                Some(user) => user,
                None => return Ok(OsuUser::Restricted { user_id }),
            }
        };
    }

    let std = get_or_restricted!(std_res, Osu);
    let tko = get_or_restricted!(tko_res, Taiko);
    let ctb = get_or_restricted!(ctb_res, Catch);
    let mna = get_or_restricted!(mna_res, Mania);

    Ok(OsuUser::Available(UserFull::new(std, tko, ctb, mna)))
}

pub async fn start_server(osu: Arc<Osu>, mysql: Database, host: String, port: u16) -> Result<()> {
    let state = ServerState::new(osu, mysql);
    let addr = format!("{host}:{port}");
    let socket_addr: SocketAddr = addr.parse().context("failed to parse server address")?;

    let app = Router::new()
        .route("/add/{user_id}", get(ServerState::add_user_handler))
        .with_state(state);

    info!("Starting local server on {addr}");

    let listener = tokio::net::TcpListener::bind(socket_addr)
        .await
        .expect("failed to bind server listener");

    tokio::select! {
        res = axum::serve(listener, app) => res?,
        _ = signal::ctrl_c() => info!("Received Ctrl+C, shutting down server"),
    }

    Ok(())
}
