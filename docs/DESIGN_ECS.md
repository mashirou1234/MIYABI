# MIYABI ECS Technical Design (Detailed)

## 1. Document Purpose and Philosophy

This document provides an exhaustive and unambiguous technical specification for the MIYABI Entity-Component-System (ECS). Its purpose is to serve as the single source of truth for the ECS implementation, eliminating guesswork and preventing architectural drift.

As requested, this design is intentionally "annoyingly detailed." Languages like Rust and C++ do not forgive ambiguity. A rigorous upfront design is the most effective way to ensure a robust, maintainable, and performant core.

The guiding philosophy remains "Code-as-Scene," but this document specifies the precise mechanics that enable that vision.

## 2. Core Data Structures: The Memory Layout of the World

### 2.1. `Entity`: The Identifier

An `Entity` is a unique, lightweight identifier for an object in the `World`.

```rust
// A simple, copyable, unique identifier for an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity(u64);
```

-   **Implementation:** A `u64` wrapper. This is the simplest possible implementation.
-   **Guarantees:** The `World` is responsible for guaranteeing that it never issues the same `Entity` ID twice.
-   **Future Consideration (Not for now):** For long-running worlds where entities are frequently created and destroyed, this could be evolved into a "generational index" to solve the "recycled ID" problem. For now, a simple incrementing counter is sufficient and correct.

### 2.2. `Component`: The Data

A `Component` is a piece of data. It must be a `struct` and contain no logic.

```rust
// The base trait for all components. 'static is required for Any-based type erasure.
// Serialize/Deserialize are needed for hot-reloading.
pub trait Component: 'static + serde::Serialize + for<'de> serde::Deserialize<'de> {
    const COMPONENT_TYPE: ComponentType;
}

// An explicit enum to identify component types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    Transform,
    Velocity,
    // ... add new component types here
}
```

-   **`'static`:** Required because we use `Box<dyn Any>` for storage, which requires owned, non-borrowed types.
-   **`Serialize`/`Deserialize`:** A non-negotiable requirement for the hot-reloading mechanism, which depends on serializing the entire `World` state.
-   **`COMPONENT_TYPE`:** We use a simple, explicit `enum` for identifying component types.
    -   **Pros:** Fast, simple, no `TypeId` hashing required for lookups.
    -   **Cons:** Requires manually adding a new variant for every new component. This is a deliberate trade-off for simplicity and performance in the early stages.
    -   **Operational rule:** When adding a new `ComponentType`, follow the single flow in [2.2.1 ComponentType Addition Checklist](#221-componenttype-addition-checklist).

### 2.2.1. ComponentType Addition Checklist

Use this checklist for every `ComponentType` addition. The target files are explicit so the implementation path stays consistent.

- [ ] **Enum variant:** Add the new variant to `ComponentType` in `logic/src/lib.rs` (`pub enum ComponentType`).
- [ ] **Component type and binding:** Define the component struct (or reuse an existing one) and add `impl Component for ... { const COMPONENT_TYPE: ... }` in `logic/src/lib.rs`.
- [ ] **Archetype storage registration:** Update `World::get_or_create_archetype` in `logic/src/lib.rs` and add the `types.contains(...)` branch that inserts `Box::new(Vec::<YourComponent>::new())` into `archetype.storage`.
- [ ] **Serialization compatibility:** Confirm the new component type derives `Serialize`/`Deserialize` and is represented safely in save/load paths (`logic/src/lib.rs` world state and `logic/src/save.rs` save pipeline).
- [ ] **Runtime readers/writers:** Search and update component-specific logic that pattern matches `ComponentType` or accesses `archetype.storage` (for example `logic/src/perf.rs`, `logic/src/ui.rs`, and system code in `logic/src/lib.rs`).
- [ ] **Query target component check:** If the component should be visible to query-based logic, verify that query candidate filtering and borrow paths (`world.query` design assumptions in section 4) still treat the new component as an explicit access target instead of relying on `HashMap` iteration side effects.
- [ ] **Design sync:** Reflect any ECS contract changes in this document and keep links valid from [3.1 Algorithm: `world.spawn()`](#31-algorithm-worldspawn) and [3.3 Algorithm: Changing an Entity (e.g., `add_component`)](#33-algorithm-changing-an-entity-eg-add_component).
- [ ] **Regression check:** Run `rg "ComponentType|COMPONENT_TYPE" logic/src/lib.rs logic/src/perf.rs logic/src/ui.rs docs/DESIGN_ECS.md` and verify a known component (for example `Button` or `Sprite`) still follows this same flow end-to-end.

### 2.3. `Archetype`: The Data Store

An `Archetype` represents a unique set of `Component` types. All entities within an archetype have the exact same components. This is the heart of our data-oriented design.

```rust
// A collection of entities that all have the same set of component types.
#[derive(Serialize, Deserialize)]
pub struct Archetype {
    // A unique, stable identifier for this archetype within the World.
    id: usize,

    // The set of component types that defines this archetype. This is its unique signature.
    types: HashSet<ComponentType>,

    // A dense, tightly packed list of all entities in this archetype.
    // The index of an entity in this Vec is its `entity_idx_in_archetype`.
    entities: Vec<Entity>,

    // The actual component data, stored in type-erased boxes.
    #[serde(skip)]
    storage: HashMap<ComponentType, Box<dyn Any>>,

    // The number of entities currently in this archetype.
    entity_count: usize,
}
```

-   **Memory Layout:** The `storage` HashMap contains the actual component data. For a given `ComponentType` (e.g., `Transform`), the corresponding `Box<dyn Any>` holds a `Vec<ffi::Transform>`. All component vectors within an archetype have the **exact same length**, which is `entity_count`. The component for `entities[i]` is located at index `i` in every `Vec<T>` in the `storage`. This guarantees a Structure-of-Arrays (SoA) layout, which is ideal for cache performance.

### 2.4. `World`: The Manager

The `World` is the public-facing struct that owns and manages all `Entity`s and `Archetype`s.

```rust
// The main container for the entire game state.
#[derive(Serialize, Deserialize)]
pub struct World {
    // The single source of truth for locating an entity.
    // Maps an Entity ID to its physical location in the archetype storage.
    entities: HashMap<Entity, EntityLocation>,

    // A dense list of all archetypes in the world.
    // The `archetype_id` in EntityLocation is an index into this Vec.
    archetypes: Vec<Archetype>,

    // Counter to generate new unique Entity IDs.
    next_entity: u64,

    // Non-serialized, transient data for the renderer.
    #[serde(skip)]
    render_commands: Vec<ffi::DrawTriangleCommand>, // To be replaced by RenderableObject
}

// The physical address of an entity's data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct EntityLocation {
    archetype_id: usize,
    // The row index of this entity within the archetype's storage vectors.
    entity_idx_in_archetype: usize,
}
```

## 3. Core Algorithms: The Precise Manipulation of State

### 3.1. Algorithm: `world.spawn()`

Spawning an entity is the process of adding a new entity and its components to the correct archetype.

**Signature:** `pub fn spawn<B: ComponentBundle>(&mut self, bundle: B) -> Entity`

1.  **Get Component Signature:** Call `B::get_component_types()` to get the `HashSet<ComponentType>` for the bundle.
2.  **Find or Create Archetype:** Call `self.get_or_create_archetype(types)` with the signature.
    -   **Internal Logic:** This function iterates through `self.archetypes`. If an `archetype.types` exactly matches the input signature, it returns its `archetype.id`.
    -   **Creation Path:** If no match is found, a new `Archetype` is created. A new `id` is assigned (which is `self.archetypes.len()`). The `types` signature is stored. Crucially, for each `ComponentType` in the signature, the `storage` map is populated with an empty, but correctly typed, `Box::new(Vec::<T>::new())`. This requires a match statement to map the enum variant to the concrete type. This is a known bottleneck for adding new components, but is explicit and required. The new archetype is pushed to `self.archetypes`, and its ID is returned. For update points when introducing a new type, use [2.2.1 ComponentType Addition Checklist](#221-componenttype-addition-checklist).
3.  **Generate Entity ID:** A new `Entity` is created using `self.next_entity`, and the counter is incremented.
4.  **Add to Archetype:**
    -   Retrieve the target `&mut Archetype` using the `archetype_id`.
    -   The `bundle.push_to_storage(archetype)` method is called. This downcasts the `Box<dyn Any>` for each component type to `&mut Vec<T>` and pushes the new component data. This operation **must never fail**, as the archetype is guaranteed to have the correct storage types.
    -   The new `Entity` is pushed to `archetype.entities`.
5.  **Update Location Map:**
    -   The new `entity_idx_in_archetype` is `archetype.entity_count - 1` (the last index).
    -   A new `EntityLocation` is created with the `archetype_id` and the new index.
    -   The pair `(new_entity, new_location)` is inserted into `world.entities`.
6.  Return the `new_entity`.

### 3.2. Algorithm: `world.despawn()` (Swap-and-Pop)

To avoid holes in our dense component arrays, we use a "swap-and-pop" strategy.

**Signature:** `pub fn despawn(&mut self, entity: Entity)`

1.  **Locate Entity:** Look up `entity` in `world.entities`. If it doesn't exist, `panic!` or return a `Result`. This gives us the `EntityLocation { archetype_id, index }`.
2.  **Get Archetype:** Retrieve `&mut self.archetypes[archetype_id]`.
3.  **Perform Swap-and-Pop:**
    -   For **each** `ComponentVec` (`Box<dyn Any>`) in `archetype.storage`:
        -   Downcast the `Box` to `&mut Vec<T>`.
        -   Call `swap_remove(index)` on the vector. This moves the last element into the slot at `index` and pops the (now duplicated) last element. It's O(1).
    -   Call `swap_remove(index)` on the `archetype.entities` vector as well.
4.  **Update Moved Entity (Crucial):**
    -   The entity that was previously at the *last* index of the archetype has now been moved to `index`. Its location has changed.
    -   Check if the removed entity was the last one (`index != archetype.entity_count - 1`).
    -   If not, get the `moved_entity_id` from `archetype.entities[index]`.
    -   Look up this `moved_entity_id` in `world.entities` and update its `EntityLocation`'s `index` field to the new `index`.
5.  **Finalize:**
    -   Decrement `archetype.entity_count`.
    -   Remove the original `entity` from the `world.entities` map.

### 3.2.1. Component Lifecycle Order (`remove` -> `reuse`)

To avoid stale references and non-deterministic behavior, structural updates must follow this exact lifecycle order when using dense vectors.

1.  **Remove first:** Execute `swap_remove(index)` on every component vector and `archetype.entities` before any insertion or reuse.
2.  **Fix moved entity location immediately:** If `swap_remove` moved the tail entity, update its `EntityLocation` in `world.entities` in the same operation.
3.  **Finalize removal state:** Decrement `entity_count` and delete the removed entity from `world.entities`.
4.  **Reuse after removal is complete:** Only after steps 1-3 are complete may the vacated slot/index be treated as reusable by subsequent `spawn`/`add_component` flows.
5.  **Do not interleave remove and reuse across archetypes:** Cross-archetype moves must complete the source-side remove sequence before target-side push/insert starts.

Operationally, this means "remove -> update moved location -> finalize -> reuse" is a single invariant and must stay consistent with [3.2. Algorithm: `world.despawn()` (Swap-and-Pop)](#32-algorithm-worlddespawn-swap-and-pop) and [3.3. Algorithm: Changing an Entity (e.g., `add_component`)](#33-algorithm-changing-an-entity-eg-add_component).

### 3.3. Algorithm: Changing an Entity (e.g., `add_component`)

This is the most complex operation, as it requires moving an entity between archetypes.

1.  **Locate Source:** Get the source `EntityLocation` and `&Archetype` for the entity.
2.  **Determine Target Signature:** Create a new `HashSet<ComponentType>` by cloning the source archetype's `types` and inserting the new component's type.
3.  **Find or Create Target Archetype:** Get the `target_archetype_id` using the new signature.
4.  **Move Component Data:**
    -   This is effectively a `despawn` from the source followed by a `spawn` to the target, but we must transfer the data.
    -   Perform a "swap-and-pop" on the **source** archetype for the entity. This returns the component data for the entity being moved. Don't forget to update the location of the swapped entity. The source side must obey [3.2.1. Component Lifecycle Order (`remove` -> `reuse`)](#321-component-lifecycle-order-remove---reuse) before any target-side reuse.
    -   Push the retrieved component data into the storage vectors of the **target** archetype.
    -   Push the *new* component into its storage vector in the target archetype. Keep this aligned with [2.2.1 ComponentType Addition Checklist](#221-componenttype-addition-checklist) so storage registration and runtime access stay consistent.
5.  **Update Location Map:** Update the `EntityLocation` for the original entity in `world.entities` to point to its new home in the target archetype.

### 3.4. Component Update Order: Operational Cautions

Runtime behavior depends not only on component data, but also on the order in which systems mutate that data. To keep results deterministic and debuggable, follow these constraints whenever adding or changing systems.

1.  **Treat system order as an explicit contract:** Define and maintain a single canonical system order in `run_logic` (or a scheduler abstraction once introduced). Do not rely on registration side effects or container iteration order.
2.  **Never depend on `HashMap` iteration order:** `archetype.storage` is a `HashMap`, and its iteration order is intentionally non-deterministic. Cross-component updates must be driven by explicit `ComponentType` access paths, not by map traversal sequence.
3.  **Single-writer rule per frame:** A given mutable component type should have one writer system per frame stage. If multiple systems need writes, split them into ordered stages and document read/write ownership.
4.  **Read-after-write ordering must be documented:** If `SystemB` assumes writes performed by `SystemA` in the same frame, that dependency must be written next to the schedule definition and reflected in this design document when changed.
5.  **Structural changes should run in a dedicated phase:** Entity moves such as `add_component`/`despawn` (swap-and-pop) can invalidate dense indices. Apply structural commands in a deferred phase after data-parallel component updates to avoid accidental stale-index access.

## 4. Query System: The Ergonomic Access Layer

The current manual iteration in `run_logic` is brittle and inefficient. A formal query system is required.

**Goal:** Provide an API like `world.query::<(&mut Transform, &Velocity)>()` that is safe, ergonomic, and performant.

### 4.1. Query Object and Iterator

A query will be a struct that holds an iterator.

```rust
pub struct Query<'w, Q> {
    world: &'w World,
    _phantom: std::marker::PhantomData<Q>,
}
```

-   The query will be parameterized by a tuple of borrows, e.g., `Q = (&'w mut Transform, &'w Velocity)`.
-   The `'w` lifetime ensures the query cannot outlive the `World` it borrows from.

### 4.2. Algorithm: `world.query()`

1.  **Request Components:** The type parameter `Q` defines the set of required components and their borrow types (`&` or `&mut`). We can use a new trait, `Queryable`, to extract this information at compile time.
2.  **Borrow Check:** The `Queryable` trait will have associated functions to check for borrow conflicts. A query for `(&mut Transform, &Transform)` is invalid. A query for `(&mut Transform, &mut Velocity)` is valid. This check happens at compile time.
3.  **Iterate Archetypes:** The query iterator will, internally:
    -   Iterate through `world.archetypes`.
    -   For each archetype, check if its `types` set is a superset of the components required by `Q`.
    -   If it is a match, it will borrow the corresponding `Vec<T>`s from the archetype's storage.
    -   It will then yield an iterator over the zipped component slices/vectors for that archetype.
4.  **Chain Iterators:** The main query object will use `iterator.chain()` to present a single, flat iterator over all matching entities across all matching archetypes.

This design leverages Rust's type system and borrow checker to provide a completely safe interface for accessing component data. The implementation will involve significant trait and lifetime magic, but the user-facing API will be clean and simple.
