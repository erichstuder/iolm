#!/usr/bin/env python3

import sys
import pathlib
from pathlib import Path
import time
import subprocess

sys.path.append(str(pathlib.Path(__file__).parent.parent / 'project_management'))
from executor import Executor # type: ignore

if __name__ == "__main__":
    additional_arguments = [
        {
            'flag': '-b',
            'name': '--build',
            'help': 'Build documentation.'
        },
        {
            'flag': '-a',
            'name': '--autobuild',
            'help': 'Start sphinx-autobuild.'
        }
    ]

    ex = Executor(additional_arguments, description='Execute unit-tests')

    if ex.arguments.build:
        commands = 'make html SPHINXOPTS="--fail-on-warning"'
    elif ex.arguments.autobuild:
        # There are some reasons why sphinx-autobuild is not used here.
        # - There are different containers where software documentation (cargo doc) and the final documentation are produced.
        #   So sphinx-autobuild cant trigger a build of the software documentation. (Except maybe with Docker-in-Docker ...)
        #   Installing rust into the doc container was refused due to duplication.
        # - Html files created with cargo doc are not hot reloadable. They are not created by sphinx-autobuild and so don't check for changes.
        #
        # commands = 'sphinx-autobuild '+ ('' if ex.arguments.verbose else '-q') +' --port 8000 --host 0.0.0.0 '
        # commands += '--watch ../software/iol/target/doc '
        # commands += '--re-ignore auto_generated source _build/html'

        def get_file_mod_times(directory, ignore):
            modification_times = {}
            for file_path in directory.rglob('*'):
                if ignore in file_path.parts:
                    continue
                modification_times[file_path] = file_path.stat().st_mtime
            return modification_times

        last_software_mod_times = {}
        last_doc_mod_times = {}
        try:
            while True:
                time.sleep(1)
                update_doc = False

                current_software_mod_times = get_file_mod_times(
                    directory=Path(__file__).parent.parent / 'software',
                    ignore='target'
                )
                if last_software_mod_times != current_software_mod_times:
                    subprocess.run([sys.executable, '../run.py', '--software', '--doc'])
                    last_software_mod_times = current_software_mod_times
                    update_doc = True

                current_doc_mod_times = get_file_mod_times(
                    directory=Path(__file__).parent / 'source',
                    ignore='auto_generated'
                )
                if last_doc_mod_times != current_doc_mod_times or update_doc:
                    subprocess.run([sys.executable, __file__, '--build'])
                    last_doc_mod_times = current_doc_mod_times
                    print('\n  -- \033[92mdocumentation was updated\033[0m (Ctrl-C to end) --')
        except KeyboardInterrupt:
            print('  Ended by user')
            exit(0)
    else:
        commands = None

    try:
        ex.run(commands)
    except KeyboardInterrupt:
        print('  Interrupted by user')
