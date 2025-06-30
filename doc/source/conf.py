# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

from sphinx.application import Sphinx
import os
import shutil
import subprocess

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
    'sphinxcontrib.images',
]

templates_path = ['_templates']

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'sphinx_rtd_theme'
html_extra_path = ['./auto_generated']

# The default export format 'svg' produced images, that had black areas when used in the html.
drawio_builder_export_format = {'html':'png'}
drawio_no_sandbox = True

def copy_rust_documentations(app: Sphinx):
    copy_pairs = [
        ('../software/examples/std/target/doc',                               'auto_generated/rust_doc/example_std'),
        ('../software/examples/stm32f446re/target/thumbv7em-none-eabihf/doc', 'auto_generated/rust_doc/example_stm32f446re'),
        ('../software/iol/target/doc',                                        'auto_generated/rust_doc/iol'),
        ('../software/iol/spec',                                              'auto_generated/rust_doc/iol/iol/spec'),
        ('../software/l6360/target/doc',                                      'auto_generated/rust_doc/l6360'),
    ]

    rust_doc_dir = os.path.join(app.srcdir, 'auto_generated', 'rust_doc')
    if os.path.exists(rust_doc_dir):
        shutil.rmtree(rust_doc_dir)

    for src, dest in copy_pairs:
        src_abs = os.path.abspath(src)
        dest_abs = os.path.join(app.srcdir, dest)
        if os.path.exists(src_abs):
            shutil.copytree(src_abs, dest_abs)
        else:
            print(f"Warning: Source directory {src_abs} does not exist.")

def render_documenations(app: Sphinx):
    dot_file = os.path.join(app.srcdir, "auto_generated", "rust_doc", "iol", "dependencies_owns.dot")
    png_file = os.path.join(app.srcdir, "auto_generated", "rust_doc", "iol", "dependencies_owns.png")
    subprocess.run(["dot", "-Tpng", "-o", png_file, dot_file], check=True)

    dot_file = os.path.join(app.srcdir, "auto_generated", "rust_doc", "iol", "dependencies_uses.dot")
    png_file = os.path.join(app.srcdir, "auto_generated", "rust_doc", "iol", "dependencies_uses.png")
    subprocess.run(["dot", "-Tpng", "-o", png_file, dot_file], check=True)

    txt_file = os.path.join(app.srcdir, "auto_generated", "rust_doc", "iol", "structure.txt")
    html_file = os.path.join(app.srcdir, "auto_generated", "rust_doc", "iol", "structure.html")
    with open(txt_file, "r") as fin, open(html_file, "w") as fout:
        subprocess.run(["ansi2html"], stdin=fin, stdout=fout, check=True)

def setup(app: Sphinx):
    app.connect("builder-inited", copy_rust_documentations)
    app.connect("builder-inited", render_documenations)
