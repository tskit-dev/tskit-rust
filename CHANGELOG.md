# Change log

## 2021-12-15, Version 0.7.0-alpha.1

Fixes various minor issues with the first alpha.

### Commits

- [[`44965969c0`](https://github.com/tskit-dev/tskit-rust/commit/44965969c04be8fa963bcaa01f11869b61d37547)] Bump version to 0.7.0.alpha.1 (molpopgen)
- [[`5d6a289490`](https://github.com/tskit-dev/tskit-rust/commit/5d6a28949087bad1632bcac41dfaefa984800113)] Fix issues with Time, Position, Location newtypes (#212) (Kevin R. Thornton)
- [[`99930e8bb0`](https://github.com/tskit-dev/tskit-rust/commit/99930e8bb013af2fc38d915b7fa3d3637bb4c847)] Add explicit tests of tree roots. (#211) (Kevin R. Thornton)


## 2021-12-15, Version 0.7.0-alpha.0

Breaking changes:

* Update to tskit C API 0.99.15.
* Add newtypes for Time, Position, and Location.
* Add newtype SizeType, which wraps tsk_size_t.
* tsk_id_t and tsk_size_t removed from library prelude.
* The Provenance trait is removed and all "provenance stuff"
  is now a first-class table API supported by the TableAccess trait.

New features:

* Update to tskit C API 0.99.15.
* tskit C structs are now managed by MBox.
* Support for virtual tree roots.
* Add postorder node iterator for Tree

### Commits

- [[`272d0736e2`](https://github.com/tskit-dev/tskit-rust/commit/272d0736e2ace79d6c4bb880977396e748b7a6a0)] bump package version to 0.7.0-alpha.0 (molpopgen)
- [[`b5946a4cc4`](https://github.com/tskit-dev/tskit-rust/commit/b5946a4cc4aec32cbdda6081a1bd188568178436)] Redesign provenance feature: (#207) (Kevin R. Thornton)
- [[`5699aab37f`](https://github.com/tskit-dev/tskit-rust/commit/5699aab37f31b25b0fdbcdb48a5f87774b624bb7)] Change return time of Tree::num_tracked_samples. (#206) (Kevin R. Thornton)
- [[`2391c9e587`](https://github.com/tskit-dev/tskit-rust/commit/2391c9e587a52638d63137017dbbd241cb5d8d8f)] Add support for virtual roots (#205) (Kevin R. Thornton)
- [[`4cf3496d82`](https://github.com/tskit-dev/tskit-rust/commit/4cf3496d8237c8115cd36b896a8b3e9096644e5d)] Fix metadata links in lib.rs (#204) (Kevin R. Thornton)
- [[`d7f70a7cfd`](https://github.com/tskit-dev/tskit-rust/commit/d7f70a7cfdb92d60d9522e7da95b2c3d0bc8b51e)] Use MBox<T> to manage lifetimes of tskit C structs. (#203) (Kevin R. Thornton)
- [[`034204ed88`](https://github.com/tskit-dev/tskit-rust/commit/034204ed8832744806063dfd3848c66f3283632c)] Add newtypes for Time, Position, and for Location. (#199) (Kevin R. Thornton)
- [[`4ba44ef0c5`](https://github.com/tskit-dev/tskit-rust/commit/4ba44ef0c5d69f660fb804194e463eb7a2db34f0)] Bump editions to 2021. (#198) (Kevin R. Thornton)
- [[`c4cc6e875a`](https://github.com/tskit-dev/tskit-rust/commit/c4cc6e875ae403963bf95ea239b6b156a62e888c)] Add tskit::SizeType (#192) (Kevin R. Thornton)
- [[`8429cc37d2`](https://github.com/tskit-dev/tskit-rust/commit/8429cc37d24fb13b7bc6e297d42794a11c5dbb8b)] Use type erasure for all table row iteration functions. (#189) (Kevin R. Thornton)
- [[`f4cb6350e0`](https://github.com/tskit-dev/tskit-rust/commit/f4cb6350e0b33deae70a2eedce26ee2403053eea)] Add TableCollection::check_integrity. (#188) (Kevin R. Thornton)
- [[`562014b04f`](https://github.com/tskit-dev/tskit-rust/commit/562014b04f24565c696088cf762f416a40335413)] Add preorder node traversal using latest tskit C API. (#186) (Kevin R. Thornton)
- [[`db8550f6d7`](https://github.com/tskit-dev/tskit-rust/commit/db8550f6d759f464203c32a32f59f4db3b118864)] Replace Box with malloc'd raw pointer for tskit types. (#184) (Kevin R. Thornton)
- [[`90a2d84a9d`](https://github.com/tskit-dev/tskit-rust/commit/90a2d84a9da1a5c59689863fdda029776fefcc44)] Update to C API 0.99.15 (breaking) (#183) (Kevin R. Thornton)
- [[`58e7c5ed73`](https://github.com/tskit-dev/tskit-rust/commit/58e7c5ed73557b1001e808d2dc5c596ebaaf60f1)] Update clap requirement from ~2.33.3 to ~2.34.0 (#176) (dependabot[bot])


## 2021-11-29, Version 0.6.1

### Commits

- [[`3eadcb161e`](https://github.com/tskit-dev/tskit-rust/commit/3eadcb161eb419dd01a4592ccc7d40bed791ceee)] Replace chrono with humantime in provenance doc tests. (#175) (Kevin R. Thornton)
- [[`4def777ddf`](https://github.com/tskit-dev/tskit-rust/commit/4def777ddf03a7a35e6300f5d955f2350dc7441f)] Add security audit work flow. (#172) (Kevin R. Thornton)

## 2021-11-23, Version 0.6.0

### Commits
- [[`b07c035077`](https://github.com/tskit-dev/tskit-rust/commit/b07c0350771b2812788dba59c5314108bf65e37c)] bump version to 0.6.0 (molpopgen)
- [[`8bb08be83d`](https://github.com/tskit-dev/tskit-rust/commit/8bb08be83d2c5a60f6aaf377b78d1cdee1d1c7e8)] Replace chrono dependency with humantime to avoid RUSTSEC-2020-0071 and RUSTSEC-2020-0159 (#171) (Momo Langenstein)
- [[`896b5891e0`](https://github.com/tskit-dev/tskit-rust/commit/896b5891e05da7f7658bd0c950ff8150c85e607c)] Implement Display for Id newtypes. (#168) (Kevin R. Thornton)
- [[`d2c6383ae9`](https://github.com/tskit-dev/tskit-rust/commit/d2c6383ae91889d1a85033e60018548503552d12)] Update to C API 0.99.14 (#165) (Kevin R. Thornton)


## 2021-09-03, Version 0.5.0

### Commits

Derive macros for table metadata.

- [[`41633b60ed`](https://github.com/tskit-dev/tskit-rust/commit/41633b60ed1ccb401a1a2794beb518ee1bc0f4ad)] Add metadata type registration via derive macros. (#163) (Kevin R. Thornton)


## 2021-08-31, Version 0.4.0

The theme of this release is "type safety".
This release breaks API due to use of newtypes for row IDs and new metadata marker traits.

### Commits

- [[`a7d78b16aa`](https://github.com/tskit-dev/tskit-rust/commit/a7d78b16aa0605b0fe07749a7cfbcbb83924010e)] Bump version to 0.4.0 (molpopogen)
- [[`b5e2c265dd`](https://github.com/tskit-dev/tskit-rust/commit/b5e2c265ddfefac6ff46273a0c588a35750d5138)] Add newtype row IDs to prelude. (#161) (Kevin R. Thornton)
- [[`3dbe8c4b5e`](https://github.com/tskit-dev/tskit-rust/commit/3dbe8c4b5eee90f0d56b096d32f48b92618e19c6)] Fix implementation of tree preorder stacking. (#160) (Kevin R. Thornton)
- [[`fafd457033`](https://github.com/tskit-dev/tskit-rust/commit/fafd45703365188cc3771cb56bb50ead220c6958)] Refactor metadata encoding: (#158) (Kevin R. Thornton)
- [[`258b4ee5f0`](https://github.com/tskit-dev/tskit-rust/commit/258b4ee5f039d643a4a6fb56ff4a8cf56919b996)] replace IdIsNull trait with associated fn (#156) (Kevin R. Thornton)
- [[`9987fc0472`](https://github.com/tskit-dev/tskit-rust/commit/9987fc0472b1f912569cdf898faa2bba9d3c5d38)] Release build optimizations: (#155) (Kevin R. Thornton)
- [[`58ac4a92d7`](https://github.com/tskit-dev/tskit-rust/commit/58ac4a92d77b7428e926c3ff942b6bf9d5872959)] Refine what "NULL"-ness means for an Id type: (#154) (Kevin R. Thornton)
- [[`ad4975fc70`](https://github.com/tskit-dev/tskit-rust/commit/ad4975fc7076057534d85e927bc39fd6e303dfb6)] Update bindgen requirement from 0.58.1 to 0.59.1 (#152) (dependabot[bot])
- [[`8a553eaa80`](https://github.com/tskit-dev/tskit-rust/commit/8a553eaa80d7872ccae3faf54a5717777bcf7e9f)] fix clippy warnings that showed up in rust 1.54 (#153) (Kevin R. Thornton)
- [[`bef4bb6c7a`](https://github.com/tskit-dev/tskit-rust/commit/bef4bb6c7acf9f054f62f2a746de1d9ed5805445)] Allow empty provenance records. (#151) (Kevin R. Thornton)
- [[`86df702040`](https://github.com/tskit-dev/tskit-rust/commit/86df7020407df7165d2bb2ae9bcff28d2abd3274)] Improve test coverage of "adding rows to table collections" (#143): (Kevin R. Thornton)
- [[`036680050b`](https://github.com/tskit-dev/tskit-rust/commit/036680050bdca732f3be4db439dfcfafd144d79c)] * Update trees API to use NodeId (#145) (Kevin R. Thornton)
- [[`7a716bbe76`](https://github.com/tskit-dev/tskit-rust/commit/7a716bbe76860fcb06d0c724e987420bd3d5048f)] Add additional member functions to TableCollection to (#141) (Kevin R. Thornton)
- [[`baf6f17430`](https://github.com/tskit-dev/tskit-rust/commit/baf6f1743015741e6438f63dc2a5122ae0739c2b)] Fix Into< > type for TableCollection::add_individual_* (#142) (Kevin R. Thornton)
- [[`689938eee7`](https://github.com/tskit-dev/tskit-rust/commit/689938eee7e07635fe07d92744c0cd24ff0bc042)] fix link from ProvenanceId to ProvenanceTable in docs (#140) (Kevin R. Thornton)
- [[`1203a5d523`](https://github.com/tskit-dev/tskit-rust/commit/1203a5d5235300782cbaeb44f32265a6ebc3340e)] strong id type cleanup (#139) (Kevin R. Thornton)
- [[`6bae100e61`](https://github.com/tskit-dev/tskit-rust/commit/6bae100e614984e2ef26e7b6b57560557a484515)] Add EdgeId (#138) (Kevin R. Thornton)
- [[`9baff077b5`](https://github.com/tskit-dev/tskit-rust/commit/9baff077b5f6dd247b6e8c01905a391ad23e5650)] Add MigrationId and ProvenanceId (#137) (Kevin R. Thornton)
- [[`7713ef669e`](https://github.com/tskit-dev/tskit-rust/commit/7713ef669e4c4e32dfb7f84f44891794a11d667b)] Add SiteId and MutationId (#136) (Kevin R. Thornton)
- [[`de006a1f0e`](https://github.com/tskit-dev/tskit-rust/commit/de006a1f0e0e0608040c344aea2e5f435f52abcd)] Add PopulationId (#135) (Kevin R. Thornton)
- [[`630ccd5f5e`](https://github.com/tskit-dev/tskit-rust/commit/630ccd5f5e9839b7fca9d3e694be01ea0b075ee2)] Add IndividualId. (#133) (Kevin R. Thornton)
- [[`8bf7d4b954`](https://github.com/tskit-dev/tskit-rust/commit/8bf7d4b9549bf0db4403e0051d43469c6a7c05d3)] NodeTable::metadata now uses Into<NodeId>. (#134) (Kevin R. Thornton)
- [[`3ac3d50f7b`](https://github.com/tskit-dev/tskit-rust/commit/3ac3d50f7b33ded239f4c19b88f3cc9a31bafe14)] * Establish a pattern for stronger ID types (#129) (Kevin R. Thornton)
- [[`e19095d9a9`](https://github.com/tskit-dev/tskit-rust/commit/e19095d9a9af035a0e91a04e8b1d9597e14f764f)] clippy vs bindgen (#130) (Kevin R. Thornton)
- [[`d9abda5bb6`](https://github.com/tskit-dev/tskit-rust/commit/d9abda5bb601b72841fc8e8e5a651f861e8e01f3)] Change links in Cargo.toml to tskit-dev (#127) (Kevin R. Thornton)

## 2021-05-19, Version 0.3.0

This release has a few API changes, etc., that streamline the public interface.

The C code is updated to 0.99.12.

### Commits

- [[`ec0adc36ae`](https://github.com/molpopgen/tskit-rust/commit/ec0adc36ae974846f61c01b4394e35017e7e4354)] streamline API for creating table row objects (#126) (Kevin R. Thornton)
- [[`edd36fa886`](https://github.com/molpopgen/tskit-rust/commit/edd36fa886839b5328b6349799b763abfd0b6ae1)] Add mutable access to NodeTable flag and time arrays. (#125) (Kevin R. Thornton)
- [[`634df568af`](https://github.com/molpopgen/tskit-rust/commit/634df568afac7a16fa319351b2d27739cb02d52d)] Fix docs for Tree::total_branch_length.  Fixes #114. (#124) (Kevin R. Thornton)
- [[`202bcded3f`](https://github.com/molpopgen/tskit-rust/commit/202bcded3f9c2bc430f51ada2b050d85877c3599)] Give access to TableCollection edge indexes. (#123) (Kevin R. Thornton)
- [[`3888fe5f10`](https://github.com/molpopgen/tskit-rust/commit/3888fe5f1090b753c4efcf57331b6d058e53f4d1)] update C code to version 0.99.12 (#122) (Kevin R. Thornton)
- [[`6fa2057af6`](https://github.com/molpopgen/tskit-rust/commit/6fa2057af6102fe35b2ae6ebd96aef7e3cf5da49)] Add prelude.rs (#119) (Kevin R. Thornton)
- [[`a592126594`](https://github.com/molpopgen/tskit-rust/commit/a59212659413892415df7e51d7c6fa0f03845828)] replace crate with \$crate for all macros (#113) (Kevin R. Thornton)
- [[`b618d75e5b`](https://github.com/molpopgen/tskit-rust/commit/b618d75e5bdc64620c27f0929111ba315080962c)] Add macro to aid returning from MetadataRoundtrip functions. (#110) (Kevin R. Thornton)
- [[`79adaa1a4a`](https://github.com/molpopgen/tskit-rust/commit/79adaa1a4a398b536d8b0fb6148703b50492aaaa)] Add macro to remove code duplication (#111) (Kevin R. Thornton)
- [[`159bbfe926`](https://github.com/molpopgen/tskit-rust/commit/159bbfe926a3efb19805ad3eabdd4a3b40cf3cec)] Add convenience macro to use all public traits. (#106) (Kevin R. Thornton)
- [[`e912f7b2c8`](https://github.com/molpopgen/tskit-rust/commit/e912f7b2c844df985b06956673432437a3264433)] Bump bindgen version. (#109) (Kevin R. Thornton)
- [[`15fae9327b`](https://github.com/molpopgen/tskit-rust/commit/15fae9327bd9adf404789d8dcdadcf32cbc4eadd)] Update clap requirement from ~2.27.0 to ~2.33.3 (#107) (dependabot[bot])
- [[`55f18a1a4f`](https://github.com/molpopgen/tskit-rust/commit/55f18a1a4f7bab301835893e8a23b01d4c01f014)] Refactor MetadataError: (#105) (Kevin R. Thornton)
- [[`fbce6ca894`](https://github.com/molpopgen/tskit-rust/commit/fbce6ca8941b4b3976d0a71b7776e2efa1fcd12a)] API fixes. (#104) (Kevin R. Thornton)


## 2021-04-27, Version 0.2.2

This release fixes issues with the lifetime relationships of `Tree` and the raw `C` arrays.
Technically, this is an API change, but only with respect to return type and the integer type used to index that return type.

### Commits
- [[`89d234f674`](https://github.com/molpopgen/tskit-rust/commit/89d234f674e8363ca6edf76f223746ae18afd283)] Replace WrappedTskArray with idiomatic slices. Closes #99 (#101) (Kevin R. Thornton)
- [[`ed8329b89e`](https://github.com/molpopgen/tskit-rust/commit/ed8329b89ec633cc32150f98c1b79eaf75ee1d83)] Completely hide NodeIterator from public name spaces. (#100) (Kevin R. Thornton)


## 2021-04-26, Version 0.2.1

- [[`ff1c7ced82`](https://github.com/molpopgen/tskit-rust/commit/ff1c7ced8260091d6fe8930ebbda3a361dac0246)] Add Provenance trait as optional feature (#98). (Kevin R. Thornton)
- [[`0fdd30067e`](https://github.com/molpopgen/tskit-rust/commit/0fdd30067e172f91a0974bbda5e2f00f1c20efe8)] Fix error in transmitting nodes. Closes #96 (#97) (Kevin R. Thornton)
- [[`299c83ae35`](https://github.com/molpopgen/tskit-rust/commit/299c83ae35516297592b61681e00e57b10de4ad4)] Update repo/homepage name in Cargo.toml (molpopgen)
- [[`35ecf29933`](https://github.com/molpopgen/tskit-rust/commit/35ecf29933506e4ee2d02b98bfc9d3d6f21d7254)] Remove re-exports that are part of existing bitflags types. (#95) (Kevin R. Thornton)


## 2021-04-21, Version 0.2.0

### Commits
- [[`daffeda43e`](https://github.com/molpopgen/tskit_rust/commit/daffeda43eeee3f2c7abe684ab6f503c943c0874)] bump version to 0.2.0 (molpopgen)
- [[`0dad5e39d2`](https://github.com/molpopgen/tskit_rust/commit/0dad5e39d2142285bac8260874a8adcb716e2e72)] Update docs (#93) (Kevin R. Thornton)
- [[`f905b9c384`](https://github.com/molpopgen/tskit_rust/commit/f905b9c3848237b53b14d45b03f19840fe62762e)] Add reverse tree iteration. (#92) (Kevin R. Thornton)
- [[`a1a033f901`](https://github.com/molpopgen/tskit_rust/commit/a1a033f901d48ee583e91b73ff30bd0797d119f2)] Add development documentation. (#91) (Kevin R. Thornton)
- [[`ce3d8f4849`](https://github.com/molpopgen/tskit_rust/commit/ce3d8f4849875650c9f3ba23522228f566bbbe2c)] Refine forward sim example (#90) (Kevin R. Thornton)
- [[`b1022e1f59`](https://github.com/molpopgen/tskit_rust/commit/b1022e1f5954dd7f1dd7caa62d7cb0ad431a32b1)] Refactor how tskit flags are handled. (#89) (Kevin R. Thornton)
- [[`56e43cb0ff`](https://github.com/molpopgen/tskit_rust/commit/56e43cb0ff045a0fb99a4ddd282b87219fde7d5d)] Add forward simulation example.  Closes #71. (#86) (Kevin R. Thornton)
- [[`5530d7b234`](https://github.com/molpopgen/tskit_rust/commit/5530d7b234fd623810a1ba717a729f228239fd9a)] Refactor all examples into examples/ (#85) (Kevin R. Thornton)
- [[`651e4edcac`](https://github.com/molpopgen/tskit_rust/commit/651e4edcac13dbc2282242fe0659d343095b5e58)] Collect public traits in traits.rs.  Closes #81. (#84) (Kevin R. Thornton)
- [[`11ad49f0e1`](https://github.com/molpopgen/tskit_rust/commit/11ad49f0e156008ae1318003dab97b84b951fc42)] Remove mention of use NodeIterator.  Closes #68. (#83) (Kevin R. Thornton)
- [[`df28231d9a`](https://github.com/molpopgen/tskit_rust/commit/df28231d9aba33a24e52279991e15a6c4f753196)] Add test fixtures module. Closes #73 (#80) (Kevin R. Thornton)
- [[`3a8a82637e`](https://github.com/molpopgen/tskit_rust/commit/3a8a82637e63d14c3e73b427ea82f4385241e320)] Rename Tree::nodes to Tree::traverse_nodes. (#79) (Kevin R. Thornton)
- [[`4e7af98ac5`](https://github.com/molpopgen/tskit_rust/commit/4e7af98ac55e3d735693c70ae41da44be8721337)] Update GitHub actions. (#78) (Kevin R. Thornton)
- [[`43e14bb8a6`](https://github.com/molpopgen/tskit_rust/commit/43e14bb8a6ebf948ba0ae386743cd9958f120d91)] Add NodeListGenerator trait bound to TableAccess.  Closes #67. (#75) (Kevin R. Thornton)
- [[`ae203769f4`](https://github.com/molpopgen/tskit_rust/commit/ae203769f404468588aac1f5043ddc79e9feaad9)] add id field to table row types. Closes #76 (#77) (Kevin R. Thornton)
- [[`346c804be8`](https://github.com/molpopgen/tskit_rust/commit/346c804be87ffa857acf22525dd6def71c6a9b38)] Make tree arrays public.  Closes #70 (#74) (Kevin R. Thornton)
- [[`a2dfcfe8f6`](https://github.com/molpopgen/tskit_rust/commit/a2dfcfe8f6e0a42f075a94529ae4d0d77f332229)] Add TableCollection and TreeSequence simplification (#64) (Kevin R. Thornton)
- [[`c62ccd21aa`](https://github.com/molpopgen/tskit_rust/commit/c62ccd21aadf32a23a8600c78431e8a6c861defb)] Add TableAccess trait and implement for TableCollection and TreeSequence. (#66) (Kevin R. Thornton)
- [[`167209466a`](https://github.com/molpopgen/tskit_rust/commit/167209466ae371a1de671f802b1f7fef5cf0fcfe)] Remove duplication of entire table collection when creating a tree sequence. (#65) (Kevin R. Thornton)
- [[`df258ad8d1`](https://github.com/molpopgen/tskit_rust/commit/df258ad8d143aec9450e236c6ebea6f487cb249e)] Fix useless conversion in lib::c_api_version (#63) (Kevin R. Thornton)
- [[`fcca8bb682`](https://github.com/molpopgen/tskit_rust/commit/fcca8bb6826f27b50de8cda397dd5584afabdc94)] remove unnecessary function from util (#62) (Kevin R. Thornton)
- [[`0ee9f86db4`](https://github.com/molpopgen/tskit_rust/commit/0ee9f86db47bbf2968e8888a7f47b791a6327c41)] Merge pull request #61 from molpopgen/add_individual_and_migration_tables (Kevin R. Thornton)
- [[`8f072c47d1`](https://github.com/molpopgen/tskit_rust/commit/8f072c47d1ece61dd55892fb64c5fa4ab2827710)] Add migrations and individuals tables. (molpopogen)
- [[`b2c891300b`](https://github.com/molpopgen/tskit_rust/commit/b2c891300b9543cbd402525a263ad5c538978289)] Merge pull request #60 from molpopgen/tskit_C_version (Kevin R. Thornton)
- [[`f48b2fed57`](https://github.com/molpopgen/tskit_rust/commit/f48b2fed575d5b08cd9cc412d9be3f9c2c490be3)] Add functions to return C API version info (molpopogen)
- [[`87593d4bdc`](https://github.com/molpopgen/tskit_rust/commit/87593d4bdc8eadd1732c20dfb3fdbff3ef370b91)] Merge pull request #59 from molpopgen/add_test_metadata_some_columns_only (Kevin R. Thornton)
- [[`5ab8e8c9d0`](https://github.com/molpopgen/tskit_rust/commit/5ab8e8c9d04c7b6f2696d0a1364c4e2f50398516)] Add test of metadata decoding when not all rows have metadata. (molpopogen)
- [[`adcc321cc8`](https://github.com/molpopgen/tskit_rust/commit/adcc321cc84d065b2e8eb0a79e88d47b9512c048)] Merge pull request #58 from molpopgen/table_row_access (Kevin R. Thornton)
- [[`860a647bf3`](https://github.com/molpopgen/tskit_rust/commit/860a647bf30eb35f60b055ac48337b83319882f9)] * Add ::row for all tables (molpopgen)
- [[`6b0a77e106`](https://github.com/molpopgen/tskit_rust/commit/6b0a77e1060b6e3a43cc06f7f6016953003e02cb)] Add util.rs (molpopgen)
- [[`6b9631e1f1`](https://github.com/molpopgen/tskit_rust/commit/6b9631e1f1d7dcd554f3b0a920101bdad54945ea)] Merge pull request #57 from molpopgen/unify_table_iteration (Kevin R. Thornton)
- [[`3d79f8bbfa`](https://github.com/molpopgen/tskit_rust/commit/3d79f8bbfaf7b555c61c68fd6c3421473022a512)] Add ability to iterate tables from a TableCollection: (molpopogen)
- [[`392ce57e0a`](https://github.com/molpopgen/tskit_rust/commit/392ce57e0a5ea26ede7788b55e2e0b0b120ccbd4)] Merge pull request #52 from molpopgen/table_iteration (Kevin R. Thornton)
- [[`2fe5912c34`](https://github.com/molpopgen/tskit_rust/commit/2fe5912c341617c4d18caecebd65c9f5127fed17)] Add table iteration w/optional handling of metadata. (molpopgen)
- [[`9a59a148f7`](https://github.com/molpopgen/tskit_rust/commit/9a59a148f74810a9c906dea5f227e07b7fac69ac)] Merge pull request #55 from molpopgen/wrapped_tsk_array_iter_adapter (Kevin R. Thornton)
- [[`84b0b496c1`](https://github.com/molpopgen/tskit_rust/commit/84b0b496c11eedf7d1ee11b1d9315ace8662ce4b)] Replace built-in iteration with iterator adapter for WrappedTskArray (molpopgen)
- [[`18234998ea`](https://github.com/molpopgen/tskit_rust/commit/18234998ea493cea7119338f176a520e16f16425)] Merge pull request #56 from molpopgen/metadata_macro_fixes (Kevin R. Thornton)
- [[`49bbd0f2f4`](https://github.com/molpopgen/tskit_rust/commit/49bbd0f2f4439a6b13d62f05c120d762bb06e5d2)] metadata_to_vector macro is now a simple return (no ? operator). (molpopgen)
- [[`6e293acf14`](https://github.com/molpopgen/tskit_rust/commit/6e293acf14a652996be45eced13f9a165c19b212)] Remove unused paramter to metadata_to_vector.  Fixes #54. (molpopgen)
- [[`45347a576c`](https://github.com/molpopgen/tskit_rust/commit/45347a576c36b15acd653efd21c8a8b3116de59b)] Change log for 0.1.2 (molpopgen)

## 2021-04-12, Version 0.1.2

### Commits
- [[`f475a5c624`](https://github.com/molpopgen/tskit_rust/commit/f475a5c6241b39690a5b18a8f2fc3af22606ad28)] Bump version to 0.1.2 (molpopgen)
- [[`f1887d55e0`](https://github.com/molpopgen/tskit_rust/commit/f1887d55e0c77c3d0cd647817cc7843711638ba2)] Merge pull request #46 from molpopgen/add_important_missing_functions (Kevin R. Thornton)
- [[`f5598256dd`](https://github.com/molpopgen/tskit_rust/commit/f5598256dd70bac04167c1eaea7f26711477036e)] Merge pull request #47 from molpopgen/add_changelog (Kevin R. Thornton)
- [[`fb85ba9606`](https://github.com/molpopgen/tskit_rust/commit/fb85ba9606258484c24f394254a0cd3d5694bd78)] Add Tree::num_tracked_samples and Tree::kc_distance. (molpopogen)
- [[`08febd5062`](https://github.com/molpopgen/tskit_rust/commit/08febd50625407a3fb0e37475180840bfd176772)] Add change log (molpopogen)
- [[`e6495a959a`](https://github.com/molpopgen/tskit_rust/commit/e6495a959a9392e01febaf8c592727432fac2609)] Merge pull request #51 from molpopgen/test_beta_instead_of_nightly (Kevin R. Thornton)
- [[`fa7cca91a5`](https://github.com/molpopgen/tskit_rust/commit/fa7cca91a5e503200e99321608257310a7d35289)] Test beta instead of nightly. (molpopgen)
- [[`bd7185eef2`](https://github.com/molpopgen/tskit_rust/commit/bd7185eef2ff6cd4f45f9ccb220c0c32c0f7e394)] Merge pull request #45 from molpopgen/documentation (Kevin R. Thornton)
- [[`2400906762`](https://github.com/molpopgen/tskit_rust/commit/24009067621b02c88547a0d6d19641e095b5d477)] Add documentation related to Tree/TreeSequence (molpopogen)
- [[`550aa76597`](https://github.com/molpopgen/tskit_rust/commit/550aa76597515b76de42d705f8bfcf1b3a3f23a4)] Fix typo in function name. (molpopogen)
- [[`56eaf875b6`](https://github.com/molpopgen/tskit_rust/commit/56eaf875b6844fd21c07bb360f437638d1b28bdf)] Merge pull request #44 from molpopgen/fix_clippy_warnings (Kevin R. Thornton)
- [[`20ffe557ca`](https://github.com/molpopgen/tskit_rust/commit/20ffe557ca1834d158151bc3a46e947f1e40a9ed)] Fix warnings from clippy (molpopogen)
- [[`0bf1dada4d`](https://github.com/molpopgen/tskit_rust/commit/0bf1dada4d8a9be5b35c04936b1b13bf423e6899)] Merge pull request #43 from molpopgen/kc_distance (Kevin R. Thornton)
- [[`2e8b4c1cfc`](https://github.com/molpopgen/tskit_rust/commit/2e8b4c1cfc3ab543453904f0ab9606d6e8becd15)] Add TreeSequence::kc_distance (molpopogen)
- [[`efe5ce2b0b`](https://github.com/molpopgen/tskit_rust/commit/efe5ce2b0b65a81dd8d45c6eb2005b3f2b2a2b3f)] Merge pull request #42 from molpopgen/fix_tskit_array_access_macro (Kevin R. Thornton)
- [[`82bb6ec6a1`](https://github.com/molpopgen/tskit_rust/commit/82bb6ec6a12ed080d9e9e0ebc5c4cd42fe14c61f)] Modify unsafe_tsk_column_access to support idiomatic fall-through. (molpopogen)
- [[`40e898910d`](https://github.com/molpopgen/tskit_rust/commit/40e898910df0e11dcb4a433c360bfc7790e260c9)] Merge pull request #41 from molpopgen/sample_list_traversal (Kevin R. Thornton)
- [[`cb0e9a4355`](https://github.com/molpopgen/tskit_rust/commit/cb0e9a435505a9f7bd7981e5a634edfcfd6ea1a6)] Add support for samples iteration: (molpopogen)
- [[`f1acfb06f2`](https://github.com/molpopgen/tskit_rust/commit/f1acfb06f2e81c81c3012eacb44da5ddce4764d4)] Merge pull request #40 from molpopgen/more_node_iteration (Kevin R. Thornton)
- [[`22684248d1`](https://github.com/molpopgen/tskit_rust/commit/22684248d10a8ad959af054f3ecb07b6a9277739)] * Rename NodeIteration to NodeIterator (molpopogen)
- [[`faa105d577`](https://github.com/molpopgen/tskit_rust/commit/faa105d5777158963af1a47ec6c8e5caa8b8384f)] Merge pull request #38 from molpopgen/Tree_API (Kevin R. Thornton)
- [[`896f921e91`](https://github.com/molpopgen/tskit_rust/commit/896f921e912a59dd5d66a9f251c9c6b5e6af8a76)] Several changes to Tree and TreeSequence: (molpopogen)
- [[`683ba84b10`](https://github.com/molpopgen/tskit_rust/commit/683ba84b1033c5a16a74faaaa080ecae5ff62949)] Merge pull request #39 from molpopgen/ffi_array_wrapper (Kevin R. Thornton)
- [[`dd3df03ca3`](https://github.com/molpopgen/tskit_rust/commit/dd3df03ca34858ee14db429bbfd2c01922509149)] Add non-owning wrappers to C arrays. (molpopogen)
- [[`85cc4a591f`](https://github.com/molpopgen/tskit_rust/commit/85cc4a591f09760d9f86ebd5cd5b4b1030254eac)] Merge pull request #37 from molpopgen/add_tree (Kevin R. Thornton)
- [[`be97490641`](https://github.com/molpopgen/tskit_rust/commit/be97490641125290e6da3372a58b1d9f4bf03b11)] Add minimial Tree interface. (molpopgen)
- [[`4b6257ed2c`](https://github.com/molpopgen/tskit_rust/commit/4b6257ed2ca52596ee8ad2f5e355ea24c3ff02bc)] Merge pull request #36 from molpopgen/streamline_macros (Kevin R. Thornton)
- [[`238f652d24`](https://github.com/molpopgen/tskit_rust/commit/238f652d249953d1449075983dd5fc5e3866dfda)] Remove macro redundancy. (molpopgen)
- [[`85505887e1`](https://github.com/molpopgen/tskit_rust/commit/85505887e16d273d46cdc4715a1f23dc331eb706)] Merge pull request #35 from molpopgen/refine_ffi (Kevin R. Thornton)
- [[`2cb4993f07`](https://github.com/molpopgen/tskit_rust/commit/2cb4993f07217fcc8ddce1472a935b594ab43f70)] Refactor the ffi module: (molpopgen)
- [[`1b9a054ff0`](https://github.com/molpopgen/tskit_rust/commit/1b9a054ff0a356d556d7719623404ab542b5f393)] Merge pull request #34 from molpopgen/improve_error_handling_macro (Kevin R. Thornton)
- [[`20a5486132`](https://github.com/molpopgen/tskit_rust/commit/20a5486132bc79e7aab9eb0e3481e8e7afbc9e73)] Add overload of handle_tsk_return_value. (molpopgen)
- [[`eae11b4c8a`](https://github.com/molpopgen/tskit_rust/commit/eae11b4c8abe3ccde66acb06ddc14856f27cd83a)] Merge pull request #33 from molpopgen/add_tree_sequence (Kevin R. Thornton)
- [[`a57b947352`](https://github.com/molpopgen/tskit_rust/commit/a57b9473522f2052a0160e0936a39986f64514c1)] Add TskitConsumingType, macro to build one, and convert TreeSequence (molpopgen)
- [[`dcb5a02f3c`](https://github.com/molpopgen/tskit_rust/commit/dcb5a02f3c998ecff7f3a05e1d1cc16d9bd3ecf8)] change dependabot to monthly (molpopgen)
- [[`4eb407ac61`](https://github.com/molpopgen/tskit_rust/commit/4eb407ac6142658984b6fc065a959c6dddbb797b)] Merge pull request #31 from molpopgen/dependabot/add-v2-config-file (Kevin R. Thornton)
- [[`a759e0441b`](https://github.com/molpopgen/tskit_rust/commit/a759e0441b2d50bd4515ff8ce3c1d7a71c681f2f)] Merge pull request #32 from molpopgen/dependabot/cargo/bindgen-0.57.0 (dependabot-preview[bot])
- [[`fb2a56c868`](https://github.com/molpopgen/tskit_rust/commit/fb2a56c868192c1a4249cff9850639c7c862dad4)] Update bindgen requirement from 0.56.0 to 0.57.0 (dependabot-preview[bot])
- [[`a8a0b1aabf`](https://github.com/molpopgen/tskit_rust/commit/a8a0b1aabf9d86c09022574ce010ff5d9682ba05)] Create Dependabot config file (dependabot-preview[bot])

## 2021-03-26, Version 0.1.1

### Commits

- [[`e9d73e4912`](https://github.com/molpopgen/tskit_rust/commit/e9d73e4912f1cac2cf5fe010f640557aa33242ed)] Fix crate name in README.md (molpopogen)
- [[`28deef7d04`](https://github.com/molpopgen/tskit_rust/commit/28deef7d04f9b64259064115f9ff899e3a0dcea8)] update README (molpopogen)
- [[`ef79a446c1`](https://github.com/molpopgen/tskit_rust/commit/ef79a446c1a02825a998b57bfba28ed157ea15bd)] Bump version to 0.1.1 (molpopogen)
- [[`d3c31fa0fc`](https://github.com/molpopgen/tskit_rust/commit/d3c31fa0fcae306baaa0b62d5403af896f52e4c4)] Merge pull request #30 from molpopgen/fix_warnings (Kevin R. Thornton)
- [[`bedfa02f76`](https://github.com/molpopgen/tskit_rust/commit/bedfa02f7616a430f6a90f686dc5845ff58d1c5e)] Merge pull request #29 from molpopgen/metadata_docs (Kevin R. Thornton)
- [[`f6f3304d75`](https://github.com/molpopgen/tskit_rust/commit/f6f3304d755a42704c1ab5af9d870d6552a2efc8)] Remove Drop constraint on TskitType. (molpopogen)
- [[`d8873a7941`](https://github.com/molpopgen/tskit_rust/commit/d8873a794163edaef7b14cf36e1691cf38f9ff69)] Quick fix regarding documenting metadata.  Closes #25 (molpopogen)
- [[`4518db4946`](https://github.com/molpopgen/tskit_rust/commit/4518db4946dfa314ac66c2a533c76e0926e5dd27)] Merge pull request #28 from molpopgen/update_to_tskit_0_3_5 (Kevin R. Thornton)
- [[`80893566ed`](https://github.com/molpopgen/tskit_rust/commit/80893566ed2445be93b8c377f0bf7a83a0d4a250)] Update C files to tskit 0.3.5 (molpopogen)
- [[`e81aad0bf6`](https://github.com/molpopgen/tskit_rust/commit/e81aad0bf6d7fc245f148923ee2bd46af01cd458)] Merge pull request #27 from molpopgen/fix_example_file_names (Kevin R. Thornton)
- [[`dbaf304b38`](https://github.com/molpopgen/tskit_rust/commit/dbaf304b3823f5343e09900cdf2170edc46a9cdf)] Fix swapped example file names. Fixes #26 (molpopogen)
- [[`30c34c2023`](https://github.com/molpopgen/tskit_rust/commit/30c34c2023ba83df7066a6bc714dd3fc2ac3848e)] Merge pull request #23 from molpopgen/rename_crate (Kevin R. Thornton)
- [[`e01b3e530e`](https://github.com/molpopgen/tskit_rust/commit/e01b3e530e3952c67a5e40d67de3a1c72cbc16e1)] rename crate to tskit (molpopgen)
- [[`a7a69ad31d`](https://github.com/molpopgen/tskit_rust/commit/a7a69ad31dd5c11a016656a1f4125e345ac5113d)] Merge pull request #20 from molpopgen/metadata (Kevin R. Thornton)
- [[`54ac4312a1`](https://github.com/molpopgen/tskit_rust/commit/54ac4312a111d542f16ca8757c8045909b345ea1)] clippy and fmt (molpopgen)
- [[`a1aea5cd95`](https://github.com/molpopgen/tskit_rust/commit/a1aea5cd95dace535e815c22a22b1062664a5c98)] add examples of metadata round trips (molpopgen)
- [[`7352ca95b5`](https://github.com/molpopgen/tskit_rust/commit/7352ca95b5708143270c90c1079f528788819eb4)] Update table operations to metadata API. (molpopgen)
- [[`275074da30`](https://github.com/molpopgen/tskit_rust/commit/275074da309646f48100c275863324f59034ad64)] Add metadata trait and API. (molpopgen)
- [[`d89dcd19dd`](https://github.com/molpopgen/tskit_rust/commit/d89dcd19dd11fa73e0d046755d08abf2f9b2f8b6)] Merge pull request #18 from molpopgen/tskit_wrapper_trait (Kevin R. Thornton)
- [[`3445af368d`](https://github.com/molpopgen/tskit_rust/commit/3445af368d8ef5cddd9eadda23063a88ec1f45df)] TableCollection uses new trait. (molpopgen)
- [[`93c43aa2c7`](https://github.com/molpopgen/tskit_rust/commit/93c43aa2c70707132610e9a7cad311ca65adf7e3)] Add trait to define what it means to wrap a tskit type. (molpopgen)
