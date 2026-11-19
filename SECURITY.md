# Security policy

## Supported versions

Weft UI is before 1.0. Security fixes go into the latest minor release only.

| Version | Supported |
| --- | --- |
| Latest 0.x minor | Yes |
| Older 0.x | No |

## Reporting a vulnerability

Do not open a public issue. Report it privately through GitHub: on the repository's **Security** tab, choose **Report a vulnerability** ([direct link](https://github.com/MioYiSama/weft-ui/security/advisories/new)).

Include the affected package and version, the platform, and steps or code that reproduce the issue. Once a fix is released, the advisory is published with credit to the reporter unless you ask otherwise.

Areas of particular interest: the native kernel and shells (`native/`), the view protocol decoders, platform calls (`fetch`, `storage`), and code the compiler generates.
