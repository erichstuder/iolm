#!/usr/bin/env python3

from project_management.executor import Executor

if __name__ == "__main__":
    additional_arguments = [
        {
            'flag': '-b',
            'name': '--build',
            'help': 'Build the software.'
        }
    ]

    ex = Executor(additional_arguments, description='Execute feature tests')

    if ex.arguments.build:
        commands = 'cargo build'
    else:
        commands = None

    ex.run(commands)
