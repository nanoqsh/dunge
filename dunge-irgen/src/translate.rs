use crate::{event::Event, func, gener};

pub(crate) fn translate<I>(events: I) -> Vec<Event>
where
    I: IntoIterator<Item = Event<func::Control>>,
{
    let mut code = vec![];
    let mut stack = vec![];

    for event in events {
        match event {
            Event::Fn(func) => code.push(Event::Fn(func)),
            Event::BlockStart => {
                stack.push(());
                code.push(Event::BlockStart);
            }
            Event::BlockEnd => {
                stack.pop();
                if !matches!(&*code, [.., Event::Semi | Event::BlockStart]) && stack.is_empty() {
                    code.push(Event::Return);
                    code.push(Event::Semi);
                }

                code.push(Event::BlockEnd);
            }
            Event::Semi => code.push(Event::Semi),
            Event::Local(local) => code.push(Event::Local(local)),
            Event::Name(name) => code.push(Event::Name(name)),
            Event::Lit(lit) => code.push(Event::Lit(lit)),
            Event::Array { len } => code.push(Event::Array { len }),
            Event::Assign => code.push(Event::Assign),
            Event::UnOp(unop) => code.push(Event::UnOp(unop)),
            Event::BinOp(binop) => code.push(Event::BinOp(binop)),
            Event::Index => code.push(Event::Index),
            Event::Member(name) => code.push(Event::Member(name)),
            Event::Method(name) => code.push(Event::Method(name)),
            Event::Call(arity) => code.push(Event::Call(arity)),
            Event::Cast(ty) => code.push(Event::Cast(ty)),
            Event::ConstExpr(ex) => code.push(Event::ConstExpr(ex)),
            Event::Return => code.push(Event::Return),
            Event::Struct(s) => code.push(Event::Struct(s)),
            Event::Control(func::Control::IfElse) => {
                code.push(Event::Control(gener::Control::IfElse));
            }
            Event::Control(func::Control::While) => {
                let block = pop(&mut code);
                let cond = pop(&mut code);
                code.push(Event::BlockStart);
                code.extend(cond);
                code.push(Event::Control(gener::Control::BreakIf));
                code.extend(block.into_iter().skip(1));
                code.push(Event::Control(gener::Control::Loop));
            }
        }
    }

    code
}

fn pop(code: &mut Vec<Event>) -> Vec<Event> {
    let mut block = 0;
    let mut take = 1;

    let mut step = code.len();
    for (backstep, event) in code.iter().enumerate().rev() {
        match event {
            Event::BlockStart => block = usize::saturating_sub(block, 1),
            Event::BlockEnd => block += 1,
            _ => {}
        }

        if block == 0 {
            take += event.arity();
            take -= 1;
            if take == 0 {
                step = backstep;
                break;
            }
        }

        step = backstep;
    }

    code.split_off(step)
}

#[cfg(test)]
mod tests {
    use {super::*, crate::event::BinOp, std::assert_matches};

    #[test]
    fn pop_empty() {
        let mut code = vec![];
        assert!(pop(&mut code).is_empty());
    }

    #[test]
    fn pop_unop() {
        let mut code = vec![Event::Semi, Event::Return, Event::Semi, Event::Return];
        assert_matches!(&*pop(&mut code), [Event::Semi, Event::Return]);
        assert_matches!(&*code, [Event::Semi, Event::Return]);
    }

    #[test]
    fn pop_binop() {
        let mut code = vec![
            Event::Semi,
            Event::Return,
            Event::Semi,
            Event::Semi,
            Event::BinOp(BinOp::Eq),
        ];

        assert_matches!(
            &*pop(&mut code),
            [Event::Semi, Event::Semi, Event::BinOp(BinOp::Eq)],
        );

        assert_matches!(&*code, [Event::Semi, Event::Return]);
    }

    #[test]
    fn pop_nested_ops() {
        let mut code = vec![
            Event::Semi,
            Event::Return,
            Event::Semi,
            Event::Semi,
            Event::Semi,
            Event::BinOp(BinOp::Eq),
            Event::BinOp(BinOp::Eq),
            Event::Semi,
            Event::BinOp(BinOp::Eq),
        ];

        assert_matches!(
            &*pop(&mut code),
            [
                Event::Semi,
                Event::Semi,
                Event::Semi,
                Event::BinOp(BinOp::Eq),
                Event::BinOp(BinOp::Eq),
                Event::Semi,
                Event::BinOp(BinOp::Eq),
            ],
        );

        assert_matches!(&*code, [Event::Semi, Event::Return]);
    }

    #[test]
    fn pop_block() {
        let mut code = vec![
            Event::Semi,
            Event::Return,
            Event::BlockStart,
            Event::BlockStart,
            Event::Semi,
            Event::BlockEnd,
            Event::BlockEnd,
        ];

        assert_matches!(
            &*pop(&mut code),
            [
                Event::BlockStart,
                Event::BlockStart,
                Event::Semi,
                Event::BlockEnd,
                Event::BlockEnd,
            ],
        );

        assert_matches!(&*code, [Event::Semi, Event::Return]);
    }

    #[test]
    fn pop_binop_blocks() {
        let mut code = vec![
            Event::Semi,
            Event::Return,
            Event::BlockStart,
            Event::Semi,
            Event::BlockEnd,
            Event::BlockStart,
            Event::Semi,
            Event::BlockStart,
            Event::Semi,
            Event::BlockEnd,
            Event::Semi,
            Event::BlockEnd,
            Event::BinOp(BinOp::Eq),
        ];

        assert_matches!(
            &*pop(&mut code),
            [
                Event::BlockStart,
                Event::Semi,
                Event::BlockEnd,
                Event::BlockStart,
                Event::Semi,
                Event::BlockStart,
                Event::Semi,
                Event::BlockEnd,
                Event::Semi,
                Event::BlockEnd,
                Event::BinOp(BinOp::Eq),
            ],
        );

        assert_matches!(&*code, [Event::Semi, Event::Return]);
    }
}
