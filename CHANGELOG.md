ChangeLog
=========

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com).

Type of changes

* Added: for new features.
* Changed: for changes in existing functionality.
* Deprecated: for soon-to-be removed features.
* Removed: for now removed features.
* Fixed: for any bug fixes.
* Security: in case of vulnerabilities.

This project adheres to [Semantic Versioning](http://semver.org).

Given a version number MAJOR.MINOR.PATCH
* MAJOR incremented for incompatible API changes
* MINOR incremented for new functionalities
* PATCH incremented for bug fixes

Additional labels for pre-release metadata:
* alpha.x: internal development stage.
* beta.x: shipped version under testing.
* rc.x: stable release candidate.

0.2.7 - 24-05-2022
--------------------
Added
* `drand` host function facilitator
* `remove_asset` host function facilitator
* `get_block_time` host function facilitator
* `get_block_time` mocked method for test
* `advanced_asset_transfer` mocked method for test


Changed
* `is_callable` now returns a boolean

0.2.7-rc1 16-02-2022
----------------
Added
* secure call host function facilitator


0.2.5 02-02-2022
----------------
Added
* hf_get_account_contract host function
* get_account_contract facilitator
* hf_is_callable host function
* is_callable facilitator


0.2.4 release skipped
----------------

Added:

* sha256 host function facilitator
* divide method to split an amount among various entities
