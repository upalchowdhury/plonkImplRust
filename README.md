# plonkImplRust

Implementing PLONK SNARK system using RUST based on the readings of plonk based literatures. Most of this code is adapted from DUSK network's plonk implementations Plus some other toy implementations such as ty_plonk. 


## components for a plonk proving system,

    1. Constraint System : Building blocks of different gates such as arithmetic gate , boolean gate, lookup gate etc 
    2. Commitment scheme : such as  KZG with SRS/CRS
    3. Transcript : Merlin
    4. Lookup table : Convenient way to look up already computed constraints instead of computing from scratch. 
    5. Prvoer : 
        Computing the proof (output will 9 ec points and 5 Fe). core items needed are , linearization poly, quotient poly, proof object and proverkey.
        There are 6 proving steps because we are using lookups:
            - Round 1, we need to compute wire polynomials and commit. This technique involves converting vectors into polynomials via Lagrange interpolation. Additionally, it’s necessary to ‘blind’ the wire polynomials. Add commitments to transcript.
            - Round 2, Derive lookup polynomials, commit and add to transcript. Blinding can be added. 
            - Round 3, Compute permutation challenges $beta$ and $gamma$. Compute $z(x)$ the permutation poly with accumultor plus blinding polys 
            - Round 4, Compute the challenge $alpha$ and quotient poly. Most compute heavy task. 
            - Round 5, Prover evaluate at a random point $zeta$
            - Round 6, Compute linearization poly. 
    6. Verifier :

