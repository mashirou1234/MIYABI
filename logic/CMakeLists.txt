# Import the Rust crate. This will define a `miyabi_logic` target.
corrosion_import_crate(
  MANIFEST_PATH ${CMAKE_CURRENT_LIST_DIR}/Cargo.toml
  FEATURES "performance_test"
)

# Create a separate target for the cxx bridge artifacts.
corrosion_add_cxxbridge(miyabi_logic_cxx
  CRATE miyabi_logic
  FILES lib.rs
)

# The generated C++ file needs to find "miyabi/bridge.h", which is in core/include.
target_include_directories(miyabi_logic_cxx PUBLIC ${CMAKE_SOURCE_DIR}/core/include)

# --- HACK: Force-clear the INTERFACE_SOURCES property ---
# The corrosion_add_cxxbridge function in this version incorrectly adds
# header files to the INTERFACE_SOURCES property, which breaks downstream
# targets. We manually clear this property as a workaround.
message(STATUS "Attempting to clear problematic INTERFACE_SOURCES from miyabi_logic_cxx")
set_target_properties(miyabi_logic_cxx PROPERTIES
    INTERFACE_SOURCES ""
)