#include "physics/PhysicsManager.hpp"
#include "miyabi_logic_cxx/lib.h" // For Vec2 and CollisionEvent definition
#include <iostream>

namespace miyabi {
namespace physics {

// --- MyContactListener Implementation ---
MyContactListener::MyContactListener(std::vector<CollisionEvent>& events)
    : m_events(events) {}

void MyContactListener::BeginContact(b2Contact* contact)
{
    b2Body* bodyA = contact->GetFixtureA()->GetBody();
    b2Body* bodyB = contact->GetFixtureB()->GetBody();

    uintptr_t body_a_id_ptr = bodyA->GetUserData().pointer;
    uintptr_t body_b_id_ptr = bodyB->GetUserData().pointer;

    // Ensure we have valid IDs before pushing an event
    if (body_a_id_ptr && body_b_id_ptr) {
        m_events.push_back({
            static_cast<PhysicsManager::BodyId>(body_a_id_ptr),
            static_cast<PhysicsManager::BodyId>(body_b_id_ptr)
        });
    }
}


// --- PhysicsManager Implementation ---
PhysicsManager::PhysicsManager() {}

PhysicsManager::~PhysicsManager() 
{
    // The world should be destroyed before the memory allocator, etc.
    // unique_ptr handles this.
}

void PhysicsManager::init()
{
    b2Vec2 gravity(0.0f, -9.8f);
    m_world = std::make_unique<b2World>(gravity);
    
    // Create and set the contact listener
    m_contact_listener = std::make_unique<MyContactListener>(m_collision_events);
    m_world->SetContactListener(m_contact_listener.get());
}

void PhysicsManager::step()
{
    if (!m_world) {
        return;
    }
    // Clear events from the previous step before the new step
    m_collision_events.clear();
    m_world->Step(m_timeStep, m_velocityIterations, m_positionIterations);
}

PhysicsManager::BodyId PhysicsManager::create_dynamic_box(float x, float y, float width, float height)
{
    if (!m_world) return 0;

    b2BodyDef bodyDef;
    bodyDef.type = b2_dynamicBody;
    bodyDef.position.Set(x, y);
    b2Body* body = m_world->CreateBody(&bodyDef);

    b2PolygonShape dynamicBox;
    dynamicBox.SetAsBox(width / 2.0f, height / 2.0f);

    b2FixtureDef fixtureDef;
    fixtureDef.shape = &dynamicBox;
    fixtureDef.density = 1.0f;
    fixtureDef.friction = 0.3f;
    body->CreateFixture(&fixtureDef);

    BodyId id = m_next_body_id++;
    m_bodies[id] = body;
    body->GetUserData().pointer = static_cast<uintptr_t>(id);
    return id;
}

PhysicsManager::BodyId PhysicsManager::create_static_box(float x, float y, float width, float height)
{
    if (!m_world) return 0;

    b2BodyDef groundBodyDef;
    groundBodyDef.position.Set(x, y);
    b2Body* groundBody = m_world->CreateBody(&groundBodyDef);

    b2PolygonShape groundBox;
    groundBox.SetAsBox(width / 2.0f, height / 2.0f);
    groundBody->CreateFixture(&groundBox, 0.0f);

    BodyId id = m_next_body_id++;
    m_bodies[id] = groundBody;
    groundBody->GetUserData().pointer = static_cast<uintptr_t>(id);
    return id;
}

Vec2 PhysicsManager::get_body_position(BodyId id)
{
    auto it = m_bodies.find(id);
    if (it != m_bodies.end()) {
        b2Vec2 position = it->second->GetPosition();
        return {position.x, position.y};
    }
    // Return a default/error value
    return {-1.0f, -1.0f};
}

const std::vector<CollisionEvent>& PhysicsManager::get_collision_events() const
{
    return m_collision_events;
}

}
}
