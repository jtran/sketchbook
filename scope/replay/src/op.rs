use crate::event::Value;
use crate::node::NId;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StepDirection {
    Forward,
    Reverse,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OpStep {
    NoOp,
    Atomic { steps: Vec<OpStep> },
    Define { id: NId },
    Forget { id: NId },
    Set { id: NId, value: Value },
    Push { id: NId, child_id: NId, value: Value },
    Pop { id: NId },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Op {
    pub forward: OpStep,
    pub reverse: OpStep,
}

impl Op {
    pub fn from_steps(forward: OpStep, reverse: OpStep, previous: OpStep) -> Op {
        let prev_rev_step = previous.rev();

        Op {
            forward: previous.then(forward),
            reverse: reverse.then(prev_rev_step),
        }
    }
    
    pub fn step_in(&self, direction: StepDirection) -> &OpStep {
        match direction {
            StepDirection::Forward => &self.forward,
            StepDirection::Reverse => &self.reverse,
        }
    }
}

impl OpStep {
    pub fn into_op(self) -> Op {
        let reverse = self.rev();

        Op {
            forward: self,
            reverse,
        }
    }

    pub fn into_op_with_previous(self, prev_step: OpStep) -> Op {
        let reverse = self.rev();
        let prev_step_rev = prev_step.rev();

        Op {
            forward: prev_step.then(self),
            reverse: reverse.then(prev_step_rev),
        }
    }

    pub fn rev(&self) -> OpStep {
        match self {
            OpStep::NoOp => OpStep::NoOp,
            OpStep::Atomic { steps } => {
                OpStep::Atomic { steps: steps.iter().rev().map(OpStep::rev).collect() }
            }
            OpStep::Define { id } => OpStep::Forget { id: *id },
            OpStep::Forget { id } => OpStep::Define { id: *id },
            OpStep::Set { .. } => panic!("OpStep is not reversible without more information: {:?}", self),
            OpStep::Push { id, .. } => OpStep::Pop { id: *id },
            OpStep::Pop { .. } => panic!("OpStep is not reversible without more information: {:?}", self),
        }
    }

    pub fn then(self, step: OpStep) -> OpStep {
        if step.is_no_op() {
            return self;
        }

        match self {
            OpStep::NoOp => step,
            OpStep::Atomic { mut steps } => {
                steps.push(step);

                OpStep::Atomic { steps }
            }
            OpStep::Define { .. }
            | OpStep::Forget { .. }
            | OpStep::Set { .. }
            | OpStep::Push { .. }
            | OpStep::Pop { .. } => {
                match step {
                    OpStep::Atomic { mut steps } => {
                        steps.insert(0, self);

                        OpStep::Atomic { steps }
                    }
                    _ => OpStep::Atomic { steps: vec![self, step] },
                }
            }
        }
    }

    pub fn is_no_op(&self) -> bool {
        *self == OpStep::NoOp
    }
}
