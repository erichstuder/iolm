#!/usr/bin/env python3

from project_management.executor import Executor # type: ignore

if __name__ == "__main__":
    additional_arguments = [
        {
            'flag': '-b',
            'name': '--build',
            'help': 'Build the software.'
        },
        {
            'flag': '-t',
            'name': '--test',
            'help': 'Test the software.'
        }
    ]

    ex = Executor(additional_arguments, description='Execute feature tests')

    if ex.arguments.build:
        commands = (
            'cd examples/stm32f446re && cargo build && cd - &&'
            'cd examples/std && cargo build && cd -'
        )

    elif ex.arguments.test:
        commands = 'cargo test --manifest-path l6360/Cargo.toml'
    else:
        commands = None

    ex.run(commands)
