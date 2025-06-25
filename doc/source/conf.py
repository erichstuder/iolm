# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

from sphinx.application import Sphinx
import os
import shutil

project = 'iolm'
copyright = '2025, erichstuder'
author = 'erichstuder'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    'sphinxcontrib.drawio',
    'sphinxcontrib.plantuml',
    'sphinx_toolbox.collapse',
    'sphinxcontrib.programoutput',
]

templates_path = ['_templates']

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'sphinx_rtd_theme'
html_extra_path = ['./auto_generated']

drawio_no_sandbox = True

def copy_cargo_doc(app: Sphinx):
    # List of (source, destination) pairs
    copy_pairs = [
        ('../software/examples/std/target/doc',                               'auto_generated/cargo_doc/example_std'),
        ('../software/examples/stm32f446re/target/thumbv7em-none-eabihf/doc', 'auto_generated/cargo_doc/example_stm32f446re'),
        ('../software/iol/target/doc',                                        'auto_generated/cargo_doc/iol'),
        ('../software/l6360/target/doc',                                      'auto_generated/cargo_doc/l6360'),
    ]

    cargo_doc_dir = os.path.join(app.srcdir, 'auto_generated', 'cargo_doc')
    if os.path.exists(cargo_doc_dir):
        shutil.rmtree(cargo_doc_dir)

    for src, dest in copy_pairs:
        src_abs = os.path.abspath(src)
        dest_abs = os.path.join(app.srcdir, dest)
        if os.path.exists(src_abs):
            shutil.copytree(src_abs, dest_abs)
        else:
            print(f"Warning: Source directory {src_abs} does not exist.")

def setup(app: Sphinx):
    app.connect("builder-inited", copy_cargo_doc)
