ARG VERSION # Should match the deployed image version

FROM us.gcr.io/logdna-k8s/vector:${VERSION}
RUN apt update && apt install -y curl git graphviz build-essential libjemalloc-dev
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN /root/.cargo/bin/cargo install --examples addr2line
RUN mkdir -p /heap
COPY jeprof.sh /root/jeprof.sh
RUN chmod +x /root/jeprof.sh

ENTRYPOINT ["/root/jeprof.sh"]
