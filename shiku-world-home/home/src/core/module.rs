use flume::{unbounded, Receiver, Sender};
use rapier2d::prelude::Real;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::core::entity::def::{EntityId, RemoveEntity, ShowEntity, UpdateEntity};
use crate::core::entity::render::{CameraSettings, ShowEffect};
use crate::core::guest::{Guest, LoginProvider, ModuleEnterSlot, ModuleExitSlot, SessionId};
use crate::core::module_system::error::CreateModuleError;
use crate::core::{blueprint, Snowflake};
use crate::resource_module::def::{GuestId, ResourceEvent, ResourceFile};
use crate::resource_module::errors::ResourceParseError;
use crate::resource_module::map::def::{LayerName, TerrainChunk};
use crate::ResourceModule;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct GuestInput {
    pub jump: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub start: bool,
    pub action_1: bool,
    pub action_2: bool,
    pub x_axis: Real,
    pub y_axis: Real,
}

impl GuestInput {
    pub fn new() -> GuestInput {
        GuestInput {
            jump: false,
            up: false,
            down: false,
            left: false,
            right: false,
            start: false,
            action_1: false,
            action_2: false,
            x_axis: 0.0,
            y_axis: 0.0,
        }
    }
}

type AuthCode = String;
type AccessToken = String;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct ProviderLoggedIn {
    pub auth_code: Option<AuthCode>,
    pub access_token: Option<AccessToken>,
    pub login_provider: LoginProvider,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum AdminToSystemEvent {
    ProviderLoggedIn(ProviderLoggedIn),
    UpdateConductor(blueprint::Conductor),
    UpdateModule(String, blueprint::ModuleUpdate),
    CreateModule(String),
    DeleteModule(String),
    SetMainDoorStatus(bool),
    SetBackDoorStatus(bool),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum GuestToModuleEvent {
    ControlInput(GuestInput),
    ResourcesLoaded(ModuleName),
    WantToChangeModule(ModuleName),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum GuestToSystemEvent {
    ProviderLoggedIn(ProviderLoggedIn),
    Ping,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum GuestTo {
    GuestToSystemEvent(GuestToSystemEvent),
    GuestToModuleEvent(GuestToModuleEvent),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum SystemToModuleEvent {
    Disconnected,
    Reconnected,
}

#[derive(Debug)]
pub enum GuestStateChange {
    ExitModule(ModuleExitSlot),
    FoundSecret(String, ModuleName),
}

#[derive(Debug)]
pub enum ModuleToSystemEvent {
    GuestStateChange(GuestStateChange),
    GlobalMessage(String),
    ToastMessage(ToastAlertLevel, String),
}

#[derive(Debug)]
pub struct GuestEvent<T> {
    pub guest_id: Snowflake,
    pub event_type: T,
}

pub enum EnterSuccessState {
    Entered,
}

#[derive(Error, Debug)]
pub enum EnterFailedState {
    #[error("Persisted state gone missing gone wild.")]
    PersistedStateGoneMissingGoneWild,
    #[error(transparent)]
    CreateModuleError(#[from] CreateModuleError),
    #[error("Guest already entered")]
    AlreadyEntered,
    #[error("Could not find game instance, wtf?")]
    GameInstanceNotFoundWTF,
}

pub enum LeaveSuccessState {
    Left,
}

#[derive(Debug)]
pub enum LeaveFailedState {
    PersistedStateGoneMissingGoneWild,
    NotInModule,
}

pub enum ModuleState {
    Starting,
    Stopped,
    /*Running,
    Stopping,
    Error,*/
}

pub type ModuleName = String;

pub trait SystemModule {
    fn module_name(&self) -> ModuleName;
    fn status(&self) -> &ModuleState;
    fn start(&mut self);
    fn shutdown(&mut self);
}

pub trait GameModule: SystemModule {
    fn get_base_resource_file(&self) -> ResourceFile;
    fn get_resource_json(&self) -> String;

    fn register_resources(
        &self,
        resource_module: &mut ResourceModule,
    ) -> Result<(), ResourceParseError> {
        resource_module.register_resources_for_module(
            self.module_name(),
            self.module_name(),
            self.get_base_resource_file(),
            Some(self.get_resource_json()),
        )?;

        Ok(())
    }
    fn update(&mut self);
    fn try_enter(
        &mut self,
        guest: &Guest,
        module_enter_slot: &ModuleEnterSlot,
    ) -> Result<EnterSuccessState, EnterFailedState>;
    fn try_leave(&mut self, guest: &Guest) -> Result<LeaveSuccessState, LeaveFailedState>;
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum SignalToGuest {
    LoginSuccess,
    LoginFailed,
}

type ShouldLogin = bool;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "bindings/Events.ts")]
pub enum CommunicationEvent {
    ResourceEvent(ResourceEvent),
    GameSystemEvent(GameSystemToGuestEvent),
    PositionEvent(Vec<(EntityId, Real, Real, Real)>),
    ConnectionReady((SessionId, ShouldLogin)),
    Signal(SignalToGuest),
    Toast(ToastAlertLevel, String),
    ShowGlobalMessage(String),
    AlreadyConnected,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum ToastAlertLevel {
    Error,
    Success,
    Info,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum MouseInputSchema {
    UpIsJumpAndNoDown,
    PurelyDirectionalNoJump,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum GameSystemToGuestEvent {
    OpenMenu(String),
    CloseMenu(String),
    UpdateDataStore(String),
    ShowTerrainChunks(Real, Vec<TerrainChunk>, ModuleName),
    SetParallax(Vec<(LayerName, (Real, Real))>),
    ShowEntities(Vec<ShowEntity>, ModuleName),
    ShowEffects(Vec<ShowEffect>, ModuleName),
    UpdateEntities(Vec<UpdateEntity>, ModuleName),
    RemoveEntities(Vec<RemoveEntity>, ModuleName),
    RemoveAllEntities(ModuleName),
    SetMouseInputSchema(MouseInputSchema),
    ChangeEntity(Vec<ShowEntity>, ModuleName),
    SetCamera(EntityId, ModuleName, CameraSettings),
}

pub type GuestToModule = GuestEvent<GuestToModuleEvent>;
pub type SystemToModule = GuestEvent<SystemToModuleEvent>;
pub type ModuleToSystem = GuestEvent<ModuleToSystemEvent>;
pub type GameSystemToGuest = GuestEvent<GameSystemToGuestEvent>;
pub type GuestToSystem = GuestEvent<GuestToSystemEvent>;
pub type GamePosition = GuestEvent<Vec<(EntityId, Real, Real, Real)>>;

pub struct ModuleIO {
    pub sender: ModuleInputSender,
    pub receiver: ModuleOutputReceiver,
}

pub struct SystemCommunicationIO {
    pub sender: Sender<(GuestId, CommunicationEvent)>,
    pub receiver: Receiver<(GuestId, CommunicationEvent)>,
}

#[derive(Clone)]
pub struct ModuleInputSender {
    pub guest_to_module_sender: Sender<GuestToModule>,
    pub system_to_module_sender: Sender<SystemToModule>,
}

pub struct ModuleOutputReceiver {
    pub module_to_system_receiver: Receiver<ModuleToSystem>,
    pub game_system_to_guest_receiver: Receiver<GameSystemToGuest>,
    pub position_receiver: Receiver<GamePosition>,
}

pub struct ModuleInputReceiver {
    pub guest_to_module_receiver: Receiver<GuestToModule>,
    pub system_to_module_receiver: Receiver<SystemToModule>,
}

#[derive(Clone)]
pub struct ModuleOutputSender {
    pub module_to_system_sender: Sender<ModuleToSystem>,
    pub game_system_to_guest_sender: Sender<GameSystemToGuest>,
    pub position_sender: Sender<GamePosition>,
}

pub fn create_module_communication() -> (
    ModuleInputSender,
    ModuleInputReceiver,
    ModuleOutputSender,
    ModuleOutputReceiver,
) {
    let (module_input_sender, module_input_receiver) = create_module_communication_input();
    let (module_output_sender, module_output_receiver) = create_module_communication_output();

    (
        module_input_sender,
        module_input_receiver,
        module_output_sender,
        module_output_receiver,
    )
}

pub fn create_module_communication_input() -> (ModuleInputSender, ModuleInputReceiver) {
    let (guest_to_module_sender, guest_to_module_receiver) = unbounded();
    let (system_to_module_sender, system_to_module_receiver) = unbounded();

    (
        ModuleInputSender {
            guest_to_module_sender,
            system_to_module_sender,
        },
        ModuleInputReceiver {
            guest_to_module_receiver,
            system_to_module_receiver,
        },
    )
}

pub fn create_module_communication_output() -> (ModuleOutputSender, ModuleOutputReceiver) {
    let (module_to_system_sender, module_to_system_receiver) = unbounded();
    let (game_system_to_guest_sender, game_system_to_guest_receiver) = unbounded();
    let (position_sender, position_receiver) = unbounded();

    (
        ModuleOutputSender {
            game_system_to_guest_sender,
            position_sender,
            module_to_system_sender,
        },
        ModuleOutputReceiver {
            game_system_to_guest_receiver,
            position_receiver,
            module_to_system_receiver,
        },
    )
}
