use super::*;

#[test]
fn every_movement_has_a_matching_direction() {
    assert_eq!(direction(Movement::ZoomIn), Direction::ZoomIn);
    assert_eq!(direction(Movement::ZoomOut), Direction::ZoomOut);
    assert_eq!(direction(Movement::LeftToRight), Direction::LeftToRight);
    assert_eq!(direction(Movement::RightToLeft), Direction::RightToLeft);
}
