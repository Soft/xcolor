FROM ekidd/rust-musl-builder

RUN sudo apt-get update && sudo apt-get install -y python python3
RUN cd /home/rust/libs && \
    curl -LO 'https://xorg.freedesktop.org/archive/individual/xcb/libxcb-1.13.tar.bz2' && \
    curl -LO 'https://xorg.freedesktop.org/archive/individual/xcb/xcb-proto-1.13.tar.bz2' && \
    curl -LO 'https://xcb.freedesktop.org/dist/libpthread-stubs-0.4.tar.gz' && \
    curl -LO 'https://www.x.org/releases/individual/lib/libXdmcp-1.1.2.tar.gz' && \
    curl -LO 'https://www.x.org/releases/individual/proto/xproto-7.0.31.tar.gz' && \
    curl -LO 'https://www.x.org/releases/individual/lib/libXau-1.0.8.tar.bz2' && \
    tar xvf libxcb-1.13.tar.bz2 && \
    tar xvf xcb-proto-1.13.tar.bz2 && \
    tar xvf libpthread-stubs-0.4.tar.gz && \
    tar xvf libXdmcp-1.1.2.tar.gz && \
    tar xvf xproto-7.0.31.tar.gz && \
    tar xvf libXau-1.0.8.tar.bz2 && \
    rm *.bz2 *.gz
RUN cd /home/rust/libs/xcb-proto-1.13 && \
    PKG_CONFIG_PATH=/usr/local/musl/lib/pkgconfig/ \
    CC=musl-gcc \
    ./configure --prefix=/usr/local/musl && \
    sudo make install
RUN cd /home/rust/libs/libpthread-stubs-0.4 && \
    PKG_CONFIG_PATH=/usr/local/musl/lib/pkgconfig/ \
    CC=musl-gcc \
    ./configure --prefix=/usr/local/musl && \
    sudo make install
RUN cd /home/rust/libs/xproto-7.0.31 && \
    PKG_CONFIG_PATH=/usr/local/musl/lib/pkgconfig/ \
    CC=musl-gcc \
    ./configure --prefix=/usr/local/musl && \
    sudo make install
RUN cd /home/rust/libs/libXdmcp-1.1.2 && \
    PKG_CONFIG_PATH=/usr/local/musl/lib/pkgconfig/ \
    CC=musl-gcc \
    ./configure --disable-shared --enable-static  --prefix=/usr/local/musl && \
    sudo make install
RUN cd /home/rust/libs/libXau-1.0.8 && \
    PKG_CONFIG_PATH=/usr/local/musl/lib/pkgconfig/ \
    CC=musl-gcc \
    ./configure --disable-shared --enable-static  --prefix=/usr/local/musl && \
    sudo make install
RUN cd /home/rust/libs/libxcb-1.13 && \
    PKG_CONFIG_PATH=/usr/local/musl/lib/pkgconfig/ \
    CC=musl-gcc \
    ./configure --disable-shared --enable-static  --prefix=/usr/local/musl && \
    sudo make install
