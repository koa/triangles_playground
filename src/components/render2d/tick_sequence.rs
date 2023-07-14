pub struct TickSequence {
    min: f64,
    max: f64,
    step: f64,
}

impl TickSequence {
    pub fn iter(&self) -> TickSequenceIterator {
        TickSequenceIterator {
            sequence: self,
            current_side: TickSequenceSide::Negative,
            last_value: Default::default(),
        }
    }
    pub fn new(min: f64, max: f64, step: f64) -> Self {
        Self { min, max, step }
    }
}

pub struct TickSequenceIterator<'a> {
    sequence: &'a TickSequence,
    current_side: TickSequenceSide,
    last_value: f64,
}

impl<'a> Iterator for TickSequenceIterator<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_side {
            TickSequenceSide::Negative => {
                let next_value = self.last_value - self.sequence.step;
                if next_value >= self.sequence.min {
                    self.last_value = next_value;
                    Some(next_value)
                } else if self.sequence.max < self.sequence.step {
                    None
                } else {
                    self.current_side = TickSequenceSide::Positive;
                    self.last_value = if self.sequence.min < 0.0 {
                        self.sequence.step
                    } else {
                        (self.sequence.min / self.sequence.step).ceil() * self.sequence.step
                    };
                    Some(self.last_value)
                }
            }
            TickSequenceSide::Positive => {
                let next_value = self.last_value + self.sequence.step;
                if next_value <= self.sequence.max {
                    self.last_value = next_value;
                    Some(self.last_value)
                } else {
                    None
                }
            }
        }
    }
}

enum TickSequenceSide {
    Negative,
    Positive,
}

#[test]
fn test_sequence() {
    let sequence = TickSequence::new(-55.0, 45.0, 5.0);
    let mut iterator = sequence.iter();
    assert_eq!(Some(-5.0), iterator.next());
    assert_eq!(Some(-10.0), iterator.next());
    assert_eq!(Some(-15.0), iterator.next());
    assert_eq!(Some(-20.0), iterator.next());
    assert_eq!(Some(-25.0), iterator.next());
    assert_eq!(Some(-30.0), iterator.next());
    assert_eq!(Some(-35.0), iterator.next());
    assert_eq!(Some(-40.0), iterator.next());
    assert_eq!(Some(-45.0), iterator.next());
    assert_eq!(Some(-50.0), iterator.next());
    assert_eq!(Some(-55.0), iterator.next());
    assert_eq!(Some(5.0), iterator.next());
}
