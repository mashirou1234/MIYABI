#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveCameraSettings {
    pub fov_degrees: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraValidationError {
    NonPositiveNearPlane,
    FarPlaneNotGreaterThanNearPlane,
    NonPositiveFov,
}

impl std::fmt::Display for CameraValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CameraValidationError::NonPositiveNearPlane => {
                write!(f, "near_plane must be > 0")
            }
            CameraValidationError::FarPlaneNotGreaterThanNearPlane => {
                write!(f, "far_plane must be > near_plane")
            }
            CameraValidationError::NonPositiveFov => write!(f, "fov_degrees must be > 0"),
        }
    }
}

impl std::error::Error for CameraValidationError {}

pub fn validate_perspective_camera_settings(
    settings: PerspectiveCameraSettings,
) -> Result<(), CameraValidationError> {
    if settings.near_plane <= 0.0 {
        return Err(CameraValidationError::NonPositiveNearPlane);
    }
    if settings.far_plane <= settings.near_plane {
        return Err(CameraValidationError::FarPlaneNotGreaterThanNearPlane);
    }
    if settings.fov_degrees <= 0.0 {
        return Err(CameraValidationError::NonPositiveFov);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_camera_settings() {
        let settings = PerspectiveCameraSettings {
            fov_degrees: 60.0,
            near_plane: 0.1,
            far_plane: 1000.0,
        };

        assert_eq!(validate_perspective_camera_settings(settings), Ok(()));
    }

    #[test]
    fn rejects_non_positive_near_plane() {
        let settings = PerspectiveCameraSettings {
            fov_degrees: 60.0,
            near_plane: 0.0,
            far_plane: 1000.0,
        };

        assert_eq!(
            validate_perspective_camera_settings(settings),
            Err(CameraValidationError::NonPositiveNearPlane)
        );
    }

    #[test]
    fn rejects_far_plane_not_greater_than_near_plane() {
        let settings = PerspectiveCameraSettings {
            fov_degrees: 60.0,
            near_plane: 1.0,
            far_plane: 1.0,
        };

        assert_eq!(
            validate_perspective_camera_settings(settings),
            Err(CameraValidationError::FarPlaneNotGreaterThanNearPlane)
        );
    }

    #[test]
    fn rejects_non_positive_fov() {
        let settings = PerspectiveCameraSettings {
            fov_degrees: -1.0,
            near_plane: 0.1,
            far_plane: 1000.0,
        };

        assert_eq!(
            validate_perspective_camera_settings(settings),
            Err(CameraValidationError::NonPositiveFov)
        );
    }
}
