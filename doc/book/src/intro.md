![Garage's Logo](img/logo.svg)

# The Garage Geo-Distributed Data Store

Garage is a lightweight geo-distributed data store.
It comes from the observation that despite numerous object stores
many people have broken data management policies (backup/replication on a single site or none at all).
To promote better data management policies, we focused on the following desirable properties:

  - **Self-contained & lightweight**: works everywhere and integrates well in existing environments to target hyperconverged infrastructures.
  - **Highly resilient**: highly resilient to network failures, network latency, disk failures, sysadmin failures.
  - **Simple**: simple to understand, simple to operate, simple to debug.
  - **Internet enabled**: made for multi-sites (eg. datacenter, offices, etc.) interconnected through a regular internet connection.

We also noted that the pursuit of some other goals are detrimental to our initial goals.
The following has been identified has non-goals, if these points matter to you, you should not use Garage:

  - **Extreme performances**: high performances constrain a lot the design and the infrastructure; we seek performances through minimalism only.
  - **Feature extensiveness**: complete implementation of the S3 API or any other API to make garage a drop-in replacement is not targeted as it could lead to decisions impacting our desirable properties.
  - **Storage optimizations**: erasure coding or any other coding technique both increase the difficulty of placing data and synchronizing; we limit ourselves to duplication.
  - **POSIX/Filesystem compatibility**: we do not aim at being POSIX compatible or to emulate any kind of filesystem. Indeed, in a distributed environment, such syncronizations are translated in network messages that impose severe constraints on the deployment.

## Supported and planned protocols

Garage speaks (or will speak) the following protocols:

  - [S3](https://docs.aws.amazon.com/AmazonS3/latest/API/Welcome.html) - *SUPPORTED* - Enable applications to store large blobs such as pictures, video, images, documents, etc. S3 is versatile enough to also be used to publish a static website.
  - [IMAP](https://github.com/go-pluto/pluto) - *PLANNED* - email storage is quite complex to get good performances.
To keep performances optimals, most imap servers only support on-disk storage.
We plan to add logic to Garage to make it a viable solution for email storage.
  - *More to come*

## Use Cases

**[Deuxfleurs](https://deuxfleurs.fr) :** Garage is used by Deuxfleurs which is a non-profit hosting organization.
Especially, it is used to host their main website, this documentation and some of its members's blogs. Additionally,
Garage is used as a [backend for Nextcloud](https://docs.nextcloud.com/server/20/admin_manual/configuration_files/primary_storage.html). Deuxfleurs also plans to use Garage as their [Matrix's media backend](https://github.com/matrix-org/synapse-s3-storage-provider) and has the backend of [OCIS](https://github.com/owncloud/ocis).

*Are you using Garage? [Open a pull request](https://git.deuxfleurs.fr/Deuxfleurs/garage/) to add your organization here!*

## Comparison to existing software

**[Minio](https://min.io/) :** Minio shares our *self-contained & lightweight* goal but selected two of our non-goals: *storage optimizations* through erasure coding and *POSIX/Filesystem compatibility* through strong consistency.
However, by pursuing these two non-goals, minio do not reach our desirable properties.
First, it fails on the *simple* property: due to the erasure coding, minio has severe limitations on how drives can be added or deleted from a cluster.
Second, it fails on the *interned enabled* property: due to its strong consistency, minio is latency sensitive.
Furthermore, minio has no knowledge of "sites" and thus can not distribute data to minimize the failure of a given site.

**[Openstack Swift](https://docs.openstack.org/swift/latest/) :**
OpenStack Swift at least fails on the *self-contained & lightweight* goal.
Starting it requires around 8Gb of RAM, which is too much especially in an hyperconverged infrastructure.
It seems also to be far from *Simple*.

**[Ceph](https://ceph.io/ceph-storage/object-storage/) :**
This review holds for the whole Ceph stack, including the RADOS paper, Ceph Object Storage module, the RADOS Gateway, etc.
At is core, Ceph has been designed to provide *POSIX/Filesystem compatibility* which requires strong consistency, which in turn
makes Ceph latency sensitive and fails our *Internet enabled* goal. 
Due to its industry oriented design, Ceph is also far from being *Simple* to operate and from being *self-contained & lightweight* which makes it hard to integrate it in an hyperconverged infrastructure.
In a certain way, Ceph and Minio are closer togethers than they are from Garage or OpenStack Swift.

*More comparisons are available in our [Related Work](design/related_work.md) chapter.*

## Other Resources

This website is not the only source of information about Garage!
We reference here other places on the Internet where you can learn more about Garage.

### Rust API (docs.rs)

If you encounter a specific bug in Garage or plan to patch it, you may jump directly to the source code documentation!

  - [garage\_api](https://docs.rs/garage_api/latest/garage_api/) - contains the S3 standard API endpoint
  - [garage\_model](https://docs.rs/garage_model/latest/garage_model/) - contains Garage's model built on the table abstraction
  - [garage\_rpc](https://docs.rs/garage_rpc/latest/garage_rpc/) - contains Garage's federation protocol
  - [garage\_table](https://docs.rs/garage_table/latest/garage_table/) - contains core Garage's CRDT datatypes
  - [garage\_util](https://docs.rs/garage_util/latest/garage_util/) - contains garage entrypoints (daemon, cli)
  - [garage\_web](https://docs.rs/garage_web/latest/garage_web/) - contains the S3 website endpoint

### Talks

We love to talk and hear about Garage, that's why we keep a log here:

  - [(fr, 2020-12-02) Garage : jouer dans la cour des grands quand on est un hébergeur associatif](https://git.deuxfleurs.fr/Deuxfleurs/garage/src/branch/master/doc/20201202_talk/talk.pdf)

*Did you write or talk about Garage? [Open a pull request](https://git.deuxfleurs.fr/Deuxfleurs/garage/) to add a link here!*

## Community

If you want to discuss with us, you can join our Matrix channel at [#garage:deuxfleurs.fr](https://matrix.to/#/#garage:deuxfleurs.fr).
Our code and our issue tracker, which is the place where you should report bugs, are managed on [Deuxfleurs' Gitea](https://git.deuxfleurs.fr/Deuxfleurs/garage).

## License

Garage, all the source code, is released under the [AGPL v3 License](https://www.gnu.org/licenses/agpl-3.0.en.html).
Please note that if you patch Garage and then use it to provide any service over a network, you must share your code!
