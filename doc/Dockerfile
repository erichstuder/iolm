FROM python:3.13.5
# Note: Can't use official sphinx image because drawio can't be installed from apt which causes problems.

RUN apt-get update && apt-get install -y \
    wget \
    libasound2 \
    plantuml \
    xvfb \
    tree \
    locales \
    # tig is great for viewing git history
    tig \
    # less is needed e.g. for 'git diff' to make the output scrollable
    less \
    graphviz

WORKDIR /usr/drawio
RUN wget https://github.com/jgraph/drawio-desktop/releases/download/v27.0.5/drawio-amd64-27.0.5.deb && \
    apt-get install -y ./drawio-amd64-27.0.5.deb

RUN python3 -m pip install --verbose \
    sphinx==8.2.3 \
    sphinx-autobuild==2024.10.3 \
    sphinxcontrib-drawio==0.0.17 \
    sphinxcontrib-plantuml==0.30 \
    sphinx-toolbox==4.0.0 \
    sphinxcontrib-programoutput==0.18 \
    pyserial==3.5 \
    sphinx-rtd-theme==3.0.2\
    setuptools==80.9.0 \
    pillow==11.2.1 \
    ansi2html==1.9.2 \
    sphinxcontrib-images==1.0.1

RUN sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen && \
    locale-gen

# Make newly created files world readable and writable.
# This is needed e.g. when cleanup up the repository from outside the container by the host user.
RUN echo "umask 0000" >> /root/.bashrc
ENV BASH_ENV=/root/.bashrc

ARG USER
ARG UID
RUN useradd -m -s /bin/bash -u ${UID:-2222} $USER
USER ${USER}
