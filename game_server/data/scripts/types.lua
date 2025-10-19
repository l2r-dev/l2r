---@alias MilliSeconds number
---@class Dictionary<T>: { [string]: T }

---@class ScriptTypeRegistration BMS

---@class ReflectReference BMS

---@class QueryResult
---@field entity fun(): Entity Entity for current query result
---@field components fun(): any[] Reference to components of Entity

---@alias Query QueryResult[]

---@class Entity ReflectReference
---@field index? fun(self): number Get entity index

---@class Vec3 Y is height
---@field x number
---@field y number
---@field z number

---@class GameVec3 Z is height
---@field x number
---@field y number
---@field z number

---@class Time bevy Time<()> resource ReflectReference
---@field delta_secs number Delta seconds from last tick

---@class SmParam
---@field variant string The type of parameter (Text, Number, Npc, Player, etc.)
---@field _1 any The value for the parameter

---@class SystemMessage
---@field id number The system message ID
---@field message_params table<number, SmParam>? Array of message parameters

---@class GameServerPacket Network packet sent from server to client
---@field variant string The packet type: "ActionFail", "Attack", "AttackStanceStart", "AttackStanceStop", "ChangeMoveType", "ChangeWaitType", "CharInfo", "CharSelectionInfo", "CreatureSay", "DeleteObject", "Die", "DropItem", "EtcStatusUpdate", "ExBasicActionList", "ExRotation", "InventoryUpdate", "ItemList", "KeyPacket", "LogoutOk", "MagicSkillLaunched", "MagicSkillCanceled", "MagicSkillUse", "MoveToLocation", "MoveToPawn", "MultisellList", and many others
---@field _1 any The packet-specific data structure

---@class BroadcastScope
---@field variant string The broadcast scope type ("KnownAndSelf", "Known", "Self", etc.)

---@class Tick
---@field tick number The tick number

---@class Transform Bevy built-in Component for spatial transformation
---@field translation table Vec3 world position coordinates {x: number, y: number, z: number}
---@field rotation table Quaternion rotation {x: number, y: number, z: number, w: number}
---@field scale table Vec3 scale factors {x: number, y: number, z: number}
--- Standard Bevy transform component used for entity positioning in 3D space

---@class StatKind Enum representing different types of character statistics
---@field variant string The stat category: "Vitals", "Attack", "Defence", "Movement", "Critical", "Primal", "ElementPower", "Inventory", "MpConsumption", "Progress", "Other"
---@field _1 any The specific stat type within the category (e.g., VitalsStat, AttackStat, etc.)

---@alias StatOp "Add"|"Mul"|"Set"|"Sub"|"Div"

---@class StatsOperation<V> Enum representing stat modification operations
---@field variant StatOp Operation type: "Set", "Add", "Sub", "Mul", "Div"
---@field _1 number The numeric value for the operation

---@class StatModifier Represents a single stat modifier with operation and priority
---@field stat StatKind The stat kind to modify
---@field operation StatsOperation The operation to apply
---@field priority number Priority for controlling application order (0 is highest)

---@class StatModifiers Bevy Component containing HashMap of named stat modifiers
---@field [string] StatModifier HashMap mapping source name to StatModifier
--- Provides methods: add_modifier, remove_modifier, remove_modifier_contains, get_top_set_modifier, apply_to_stat, merge, unmerge

---@class EulerRot
---@field variant string Rotation order ("YXZ", "XYZ", etc.)

---@class ClassId Bevy Component for character class identifier
---@field _1 number The class ID value

---@class BaseClass Bevy Component for character's base class
---@field _1 number The base class ID value

---@class SubClass Bevy Component for character's subclass (optional)
---@field _1 number The subclass ID value

---@class SubClassVariant Enum representing which subclass slot this skill belongs to
---@field variant string The variant name: "Main", "SubClass1", "SubClass2", "SubClass3"

---@class SkillId
---@field _1 number

---@class SkillLevel
---@field _1 number

---@class Skill ReflectReference
---@field id SkillId Skill ID
---@field display_id number? Optional display ID for skill representation
---@field level SkillLevel Skill level - values > 100 indicate enchanted skills
---@field kind SkillKindReference Skill kind (e.g., "Active", "Passive", "Toggle")
---@field magic_level number Magic level of the skill
---@field disabled boolean Whether the skill is disabled

---@class LearnRequirement Enum representing skill learning requirements
---@field variant string Requirement type: "Auto", "Level", "Sp", "Item"
---@field _1 any? The requirement value (Level number, Sp number, or Item tuple [item_id, count])
---@field variant_name fun(self): string Get the variant name of this requirement

---@class LearnRequirementsArray Reflected Vec of LearnRequirement
---@field [number] LearnRequirement Indexed access to requirements (1-based)
---@field len fun(self): number Get length of requirements array

---@class LearnRequirements Collection of learning requirements for a skill (Vec wrapper)
---@field _1 LearnRequirementsArray Array of LearnRequirement objects with reflection methods

---@class SkillTreeNode Represents a node in the skill tree
---@field skill_id SkillId The skill ID
---@field skill_level SkillLevel The skill level
---@field requirements LearnRequirements Requirements to learn this skill

---@class SkillTreeNodes Reflected Vec wrapper for skill tree nodes
---@field [number] SkillTreeNode Indexed access to skill tree nodes (1-based)
---@field len fun(self): number Get length of skill tree nodes array

---@class SkillTree Asset containing skill tree data
---@field _1 SkillTreeNodes Array of SkillTreeNode objects (Vec<SkillTreeNode>)

---@class SkillTreesHashMap: table<number, any>
---@field map_get fun(self, key: number): any? Get handle by class ID
---@field insert fun(self, key: number, value: any) Insert or update handle
---@field remove fun(self, key: number): any? Remove and return handle by class ID

---@class SkillTreesHandlers Resource containing handles to skill trees by class ID
---@field _1 SkillTreesHashMap HashMap mapping ClassId to Handle<SkillTree>

---@class DollSlot Equipment slot identifier for character paperdoll
---@field variant string The equipment slot type: "Underwear", "Head", "AccessoryLeft", "AccessoryRight", "Neck", "RightHand", "Chest", "LeftHand", "RightEar", "LeftEar", "Gloves", "Legs", "Feet", "RightFinger", "LeftFinger", "LeftBracelet", "RightBracelet", "Talisman1"-"Talisman6", "Cloak", "Belt"

---@class PaperDoll Bevy Component for character equipment management
---@field slots table<DollSlot, any> HashMap mapping equipment slots to optional unique items
--- Used for tracking equipped items, calculating base defense values, and managing equipment changes

---@class ItemsDataTable Resource containing item data
---@field _1 table The data table

---@class ItemsInfo Asset containing item information

---@class DynamicComponent
---@field data table The component data

---@class TimerMode Bevy Timer mode enum controlling timer behavior
---@field variant string Timer mode: \"Once\" (ticks down once, resets manually) or \"Repeat\" (auto-resets when reaching 0)

---@class AnimationTimer Bevy Component wrapping a Timer for animation duration tracking
---@field _1 table Timer data with duration and mode (typically TimerMode::Once)
--- Uses SparseSet storage, requires Animation component, implements Deref/DerefMut for Timer access

---@class SkillListHashMap: table<number, Skill>
---@field map_get fun(self, key: SkillId): Skill? Get skill by ID
---@field insert fun(self, key: SkillId, value: Skill) Insert or update skill
---@field remove fun(self, key: SkillId): Skill? Remove and return skill by ID

---@class SkillList Bevy Component containing character's learned skills
---@field _1 SkillListHashMap HashMap mapping skill IDs to Skill objects (skills::Id -> Skill)
--- Provides methods: map_get(key), insert(key, value), remove(key). Implements Deref/DerefMut for HashMap access.

---@class AbnormalEffectTimer Timer and effects over time data for a single abnormal effect
---@field timer table? Optional Timer for duration-based effects (nil for infinite duration)
---@field effects_over_time table? Optional array of EffectOverTime objects for periodic effects

---@class AbnormalEffectsTimersHashMap: table<number, AbnormalEffectTimer>
---@field map_get fun(self, key: SkillId): AbnormalEffectTimer? Get timer by skill ID
---@field insert fun(self, key: SkillId, value: AbnormalEffectTimer) Insert or update timer
---@field remove fun(self, key: SkillId): AbnormalEffectTimer? Remove and return timer by skill ID

---@class AbnormalEffectsTimers Bevy Component containing timers for abnormal effects
---@field _1 AbnormalEffectsTimersHashMap HashMap mapping skill IDs to AbnormalEffectTimer objects
--- Automatically required by AbnormalEffects component. Manages timers and effects over time separately from effect state.

---@class ObjectId Bevy Component with immutable unique object identifier
---@field _1 number The object ID (u32) - managed by ObjectIdManager for entity tracking
--- Uses Table storage type and is immutable once assigned

---@class Animation Bevy Component for entity animation state
--- Uses SparseSet storage and is immutable. Triggers AnimationFinished event when removed.
--- Actual animation data is managed by the animation system.

---@class KnownEntities Bevy Component tracking entities known to this entity
---@field entities table<number, Entity> EntityHashSet of entities within perception range
--- Uses Table storage, triggers cleanup when removed, used for networking and AI awareness

-- Plugin System Types --
---@class AppStats
---@field plugins_count integer Number of registered plugins
---@field initialized boolean Whether the app has been initialized

---@class LuaApp
---@field app_name string The name of the application
---@field plugins PluginRegistry Map of registered plugins by name
---@field _initialized boolean Whether the app has been initialized
