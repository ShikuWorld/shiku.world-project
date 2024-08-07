use flume::{unbounded, Receiver, Sender};
use rapier2d::prelude::Real;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::core::blueprint;
use crate::core::blueprint::character_animation::CharacterAnimation;
use crate::core::blueprint::def::{
    CharAnimationToTilesetMap, Chunk, Conductor, Gid, GidMap, Image, LayerKind, ModuleId,
    ResourcePath, TerrainParams, Tile, Tileset,
};
use crate::core::blueprint::ecs::def::{Entity, EntityUpdate, EntityUpdateKind};
use crate::core::blueprint::scene::def::{
    CollisionShape, GameNodeId, GameNodeKind, Scene, SceneId, Script,
};
use crate::core::entity::def::EntityId;
use crate::core::entity::render::CameraSettings;
use crate::core::guest::{ActorId, LoginProvider, ModuleExitSlot, SessionId};
use crate::core::module_system::game_instance::GameInstanceId;
use crate::core::module_system::world::WorldId;
use crate::resource_module::def::{ResourceBundle, ResourceEvent};

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
pub enum EditorEvent {
    Modules(Vec<blueprint::def::Module>),
    ModuleInstances(Vec<(ModuleId, Vec<GameInstanceId>)>),
    CreatedModule(ModuleId, blueprint::def::Module),
    DeletedModule(ModuleId),
    UpdatedModule(ModuleId, blueprint::def::Module),
    CreatedScript(Script),
    SetScript(Script),
    DeletedScript(Script),
    CreatedMap(blueprint::def::GameMap),
    SetMap(blueprint::def::GameMap),
    UpdatedMap(blueprint::def::MapUpdate),
    DeletedMap(blueprint::def::GameMap),
    CreatedScene(blueprint::scene::def::Scene),
    SetScene(blueprint::scene::def::Scene),
    UpdateScene(SceneNodeUpdate),
    DeletedScene(blueprint::scene::def::Scene),
    CreatedTileset(blueprint::def::Tileset),
    SetTileset(blueprint::def::Tileset),
    DeletedTileset(blueprint::def::Tileset),
    CreatedCharacterAnimation(CharacterAnimation),
    SetCharacterAnimation(CharacterAnimation),
    DeletedCharacterAnimation(CharacterAnimation),
    DirectoryInfo(blueprint::def::FileBrowserResult),
    UpdatedConductor(Conductor),
    ModuleInstanceOpened(ModuleId, GameInstanceId),
    ModuleInstanceClosed(ModuleId, GameInstanceId),
    MainDoorStatus(bool),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum AdminToSystemEvent {
    ProviderLoggedIn(ProviderLoggedIn),
    UpdateConductor(Conductor),
    BrowseFolder(String),
    OpenInstance(ModuleId),
    StartInspectingWorld(ModuleId, GameInstanceId, WorldId),
    StopInspectingWorld(ModuleId, GameInstanceId, WorldId),
    ControlInput(ModuleId, GameInstanceId, GuestInput),
    WorldInitialized(ModuleId, GameInstanceId, WorldId),
    UpdateModule(ModuleId, blueprint::def::ModuleUpdate),
    CreateModule(ModuleName),
    GetResource(ResourcePath),
    CreateTileset(ModuleId, Tileset),
    SetTileset(Tileset),
    UpdateTileset(ResourcePath, TilesetUpdate),
    DeleteTileset(Tileset),
    CreateScene(ModuleId, Scene),
    UpdateSceneNode(SceneNodeUpdate),
    UpdateInstancedNode(ModuleId, GameInstanceId, WorldId, EntityUpdate),
    ResetGameWorld(ModuleId, GameInstanceId, WorldId),
    OverwriteSceneRoot(ResourcePath, GameNodeKind),
    RemoveInstanceNode(ModuleId, GameInstanceId, WorldId, Entity),
    AddNodeToInstanceNode(ModuleId, GameInstanceId, WorldId, Entity, GameNodeKind),
    DeleteScene(Scene),
    CreateMap(ModuleId, blueprint::def::GameMap),
    UpdateMap(blueprint::def::MapUpdate),
    DeleteMap(ModuleId, blueprint::def::GameMap),
    CreateScript(ModuleId, Script),
    UpdateScript(Script),
    DeleteScript(Script),
    CreateCharacterAnimation(ModuleId, CharacterAnimation),
    UpdateCharacterAnimation(CharacterAnimation),
    DeleteCharacterAnimation(CharacterAnimation),
    DeleteModule(ModuleId),
    SetMainDoorStatus(bool),
    SetBackDoorStatus(bool),
    LoadEditorData,
    Ping,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum SceneNodeUpdate {
    UpdateData(SceneId, Vec<usize>, GameNodeId, EntityUpdateKind),
    AddChild(SceneId, Vec<usize>, GameNodeId, GameNodeKind),
    RemoveChild(SceneId, Vec<usize>, GameNodeKind),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum TilesetUpdate {
    AddTile(Gid, Tile),
    RemoveTile(Gid),
    ChangeTileImage(Gid, Image),
    UpdateCollisionShape(Gid, CollisionShape),
    RemoveCollisionShape(Gid),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum GuestToModuleEvent {
    ControlInput(GuestInput),
    GameSetupDone,
    WantToChangeModule(Option<ModuleExitSlot>),
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
    Disconnected(ActorId),
    Reconnected(ActorId),
}

#[derive(Debug)]
pub enum GuestStateChange {
    ExitModule(ModuleExitSlot),
    FoundSecret(String, ModuleName),
}

#[derive(Debug)]
pub enum ModuleToSystemEvent {
    GuestStateChange(ActorId, GuestStateChange),
    GameInstanceCreated(ModuleId, GameInstanceId),
    GameInstanceClosed(ModuleId, GameInstanceId),
    GlobalMessage(String),
    ToastMessage(ActorId, ToastAlertLevel, String),
}

#[derive(Debug)]
pub struct GuestEvent<T> {
    pub guest_id: ActorId,
    pub event_type: T,
}
#[derive(Debug, Clone)]
pub struct ModuleInstanceEvent<T> {
    pub module_id: ModuleId,
    pub instance_id: GameInstanceId,
    pub world_id: Option<WorldId>,
    pub event_type: T,
}

pub enum EnterSuccessState {
    Entered,
}

#[derive(Debug)]
pub enum AdminEnterSuccessState {
    EnteredWorld,
    EnteredInstanceAndWorld,
    AlreadyEntered,
}

#[derive(Debug)]
pub enum AdminLeftSuccessState {
    LeftWorld,
    LeftWorldAndInstance,
}

#[derive(Error, Debug)]
pub enum EnterFailedState {
    #[error("Persisted state gone missing gone wild.")]
    PersistedStateGoneMissingGoneWild,
    #[error("Already entered")]
    AlreadyEntered,
    #[error("No Main map set")]
    NoMainMapSet,
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
    fn start(&mut self);
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum SignalToMedium {
    LoginSuccess,
    LoginFailed,
}

type ShouldLogin = bool;
type IsMainInstance = bool;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "bindings/Events.ts")]
pub enum CommunicationEvent {
    ResourceEvent(ModuleId, ResourceEvent),
    PrepareGame(
        ModuleId,
        GameInstanceId,
        Option<WorldId>,
        ResourceBundle,
        TerrainParams,
        Vec<(LayerKind, f32, f32)>,
        Vec<Tileset>,
        GidMap,
        CharAnimationToTilesetMap,
    ),
    UnloadGame(ModuleId, GameInstanceId, Option<WorldId>),
    GameSystemEvent(
        ModuleId,
        GameInstanceId,
        Option<WorldId>,
        GameSystemToGuestEvent,
    ),
    ConnectionReady((SessionId, ShouldLogin)),
    Signal(SignalToMedium),
    Toast(ToastAlertLevel, String),
    ShowGlobalMessage(String),
    AlreadyConnected,
    EditorEvent(EditorEvent),
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
    ShowTerrain(Vec<(LayerKind, Vec<Chunk>)>),
    SetParallax(Vec<(LayerKind, f32, f32)>),
    ShowTerrainCollisionLines(Vec<Vec<(Real, Real)>>),
    ShowScene(Scene),
    UpdateModuleMaps((GidMap, CharAnimationToTilesetMap)),
    UpdateEntity(EntityUpdate),
    RemoveEntity(Entity),
    AddEntity(Entity, GameNodeKind),
    SetMouseInputSchema(MouseInputSchema),
    SetCamera(EntityId, CameraSettings),
    PositionEvent(Vec<(Entity, Real, Real, Real)>),
}

pub type GuestToModule = GuestEvent<ModuleInstanceEvent<GuestToModuleEvent>>;
pub type SystemToModule = ModuleInstanceEvent<SystemToModuleEvent>;
pub type ModuleToSystem = ModuleToSystemEvent;
pub type GameSystemToGuest = GuestEvent<ModuleInstanceEvent<GameSystemToGuestEvent>>;
pub type GamePosition = GuestEvent<(
    ModuleName,
    GameInstanceId,
    Option<WorldId>,
    Vec<(EntityId, Real, Real, Real)>,
)>;

pub struct ModuleIO {
    pub sender: ModuleInputSender,
    pub receiver: ModuleOutputReceiver,
}

pub struct SystemCommunicationIO {
    pub sender: Sender<(ActorId, CommunicationEvent)>,
    pub receiver: Receiver<(ActorId, CommunicationEvent)>,
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
