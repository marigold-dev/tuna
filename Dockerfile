FROM ocaml/opam

WORKDIR /app

COPY packages dune dune-project tunac.opam tunac.opam.locked .

RUN sudo apt update -y && \
    sudo apt install -y libgmp-dev pkg-config && \
    opam install . --deps-only

RUN eval $(opam env) && dune build --release

RUN sudo install  ./_build/install/default/bin/tunac /usr/bin/tunac