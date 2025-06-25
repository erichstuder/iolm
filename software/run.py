#!/usr/bin/env python3

import sys
import pathlib

sys.path.append(str(pathlib.Path(__file__).parent.parent / 'project_management'))
from executor import Executor # type: ignore

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
        },
        {
            'flag': '-d',
            'name': '--doc',
            'help': 'Document the software.'
        }
    ]

    ex = Executor(additional_arguments, description='Execute feature tests')

    if ex.arguments.build:
        commands = (
            'cd examples/std && cargo build && cd - &&'
            'cd examples/stm32f446re && cargo build && cd -'
        )
    elif ex.arguments.test:
        commands = (
            'cargo test --manifest-path l6360/Cargo.toml &&'
            # Note: log feature is not specifically tested and always enabled so it compiles.
            'cargo test --manifest-path iol/Cargo.toml --features "log master" &&'
            'cargo test --manifest-path iol/Cargo.toml --features "log master iols"'
        )
    elif ex.arguments.doc:
        commands = (
            'cd examples/std && cargo doc --no-deps && cd - &&'
            'cd examples/stm32f446re && cargo doc --no-deps && cd - &&'
            'cargo doc --manifest-path l6360/Cargo.toml --no-deps &&'
            'cargo doc --manifest-path iol/Cargo.toml --no-deps '
        )
    else:
        commands = None

    ex.run(commands)
