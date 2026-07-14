/// Macro to attach a function with both entry and return probes at a resolved
/// file offset. Also generates an `attach_*_if_found` variant that skips
/// symbols absent from the offset table (returning whether it attached) and
/// propagates attach failures.
macro_rules! attach_uprobe_uretprobe {
    ($name:ident, $prog_entry:ident, $prog_return:ident) => {
        paste! {
            fn [<try_ $name>](&mut self, lib_path: &Path, offset: usize) -> Result<()> {
                let link = self
                    .skel
                    .progs
                    .$prog_entry
                    .attach_uprobe_with_opts(
                        -1,
                        lib_path,
                        offset,
                        UprobeOpts {
                            retprobe: false,
                            ..Default::default()
                        },
                    )
                    .context(format!(
                        "Failed to attach uprobe at offset {:#x} in {}",
                        offset,
                        lib_path.display()
                    ))?;
                self.probes.push(link);

                let link = self
                    .skel
                    .progs
                    .$prog_return
                    .attach_uprobe_with_opts(
                        -1,
                        lib_path,
                        offset,
                        UprobeOpts {
                            retprobe: true,
                            ..Default::default()
                        },
                    )
                    .context(format!(
                        "Failed to attach uretprobe at offset {:#x} in {}",
                        offset,
                        lib_path.display()
                    ))?;
                self.probes.push(link);

                Ok(())
            }

            fn [<$name _if_found>](
                &mut self,
                lib_path: &Path,
                symbol: &str,
                symbols: &ResolvedSymbols,
            ) -> Result<bool> {
                let Some(offset) = symbols.offset(symbol) else {
                    return Ok(false);
                };
                self.[<try_ $name>](lib_path, offset)
                    .with_context(|| format!("Failed to attach {symbol}"))?;
                log::trace!("Attached {} at {:#x}", symbol, offset);
                Ok(true)
            }
        }
    };
}

/// Macro to attach a function with only an entry probe (no return probe) at a
/// resolved file offset. Also generates an `attach_*_if_found` variant that
/// skips symbols absent from the offset table (returning whether it attached)
/// and propagates attach failures.
macro_rules! attach_uprobe {
    ($name:ident, $prog:ident) => {
        paste! {
            fn [<try_ $name>](&mut self, lib_path: &Path, offset: usize) -> Result<()> {
                let link = self
                    .skel
                    .progs
                    .$prog
                    .attach_uprobe_with_opts(
                        -1,
                        lib_path,
                        offset,
                        UprobeOpts {
                            retprobe: false,
                            ..Default::default()
                        },
                    )
                    .context(format!(
                        "Failed to attach uprobe at offset {:#x} in {}",
                        offset,
                        lib_path.display()
                    ))?;
                self.probes.push(link);
                Ok(())
            }

            fn [<$name _if_found>](
                &mut self,
                lib_path: &Path,
                symbol: &str,
                symbols: &ResolvedSymbols,
            ) -> Result<bool> {
                let Some(offset) = symbols.offset(symbol) else {
                    return Ok(false);
                };
                self.[<try_ $name>](lib_path, offset)
                    .with_context(|| format!("Failed to attach {symbol}"))?;
                log::trace!("Attached {} at {:#x}", symbol, offset);
                Ok(true)
            }
        }
    };
}

macro_rules! attach_tracepoint {
    ($func:ident, $prog:ident) => {
        fn $func(&mut self) -> Result<()> {
            let link = self
                .skel
                .progs
                .$prog
                .attach()
                .context(format!("Failed to attach {} tracepoint", stringify!($prog)))?;
            self.probes.push(link);
            Ok(())
        }
    };
    ($name:ident) => {
        paste! {
            attach_tracepoint!([<attach_ $name>], [<tracepoint_ $name>]);
        }
    };
}
