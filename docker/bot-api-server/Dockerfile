FROM debian:bookworm AS builder

WORKDIR /build

RUN apt-get update && apt-get upgrade -y && apt-get install -y \
    make \
    git \
    zlib1g-dev \
    libssl-dev \
    gperf \
    cmake \
    clang \
    libc++-dev \
    libc++abi-dev \
    && apt-get clean

RUN git clone --recursive https://github.com/tdlib/telegram-bot-api.git

RUN cd telegram-bot-api \
    && rm -rf build \
    && mkdir build \
    && cd build \
    && CXXFLAGS="-stdlib=libc++" CC=/usr/bin/clang CXX=/usr/bin/clang++ cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX:PATH=/usr/local .. \
    && cmake --build . --target install

FROM debian:bookworm AS runtime

COPY --from=builder /usr/local/bin/telegram-bot-api /usr/local/bin/telegram-bot-api

RUN apt-get update && apt-get install -y \
    libc++1 \
    libc++abi1 \
    openssl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* \

EXPOSE 8081

ENTRYPOINT ["/usr/local/bin/telegram-bot-api"]
