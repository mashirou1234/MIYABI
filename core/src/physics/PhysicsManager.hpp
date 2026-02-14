#pragma once

#include "miyabi/bridge.h" // For Vec2 and CollisionEvent forward declaration
#include <box2d/box2d.h>
#include <memory>
#include <vector>
#include <unordered_map>
#include <cstdint>

// Forward declare to avoid including box2d headers in other files
class b2Contact;

namespace miyabi {
namespace physics {

class MyContactListener : public b2ContactListener
{
public:
    MyContactListener(std::vector<CollisionEvent>& events);
    void BeginContact(b2Contact* contact) override;

private:
    std::vector<CollisionEvent>& m_events;
};

class PhysicsManager
{
public:
    using BodyId = uint64_t;

    PhysicsManager();
    ~PhysicsManager();

    void init();
    void step();

    BodyId create_dynamic_box(float x, float y, float width, float height);
    BodyId create_static_box(float x, float y, float width, float height);
    Vec2 get_body_position(BodyId id);
    const std::vector<CollisionEvent>& get_collision_events() const;


private:
    std::unique_ptr<b2World> m_world;
    std::unique_ptr<MyContactListener> m_contact_listener;
    std::unordered_map<BodyId, b2Body*> m_bodies;
    std::vector<CollisionEvent> m_collision_events;
    BodyId m_next_body_id = 1;

    const float m_timeStep = 1.0f / 60.0f;
    const int32_t m_velocityIterations = 6;
    const int32_t m_positionIterations = 2;
};

}
}
