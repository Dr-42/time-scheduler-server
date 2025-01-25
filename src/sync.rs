use std::collections::HashMap;
use std::path::Path;

use chrono::{NaiveDate, NaiveTime};
use indexmap::IndexMap;

use crate::blocktype::BlockType;
use crate::currentblock::CurrentBlock;
use crate::err::{Error, ErrorType};
use crate::err_from_type;
use crate::timeblock::TimeBlock;

// Define a trait `Sync` that provides an asynchronous `sync` function for synchronizing data.
pub trait Sync<T> {
    async fn sync(data_dir: &Path, incoming_data: T) -> Result<(), Error>;
}

// Define a type alias for a vector of `BlockType` objects.
type SyncBlockType = Vec<BlockType>;

// Implement the `Sync` trait for `BlockType` to handle syncing block types.
impl Sync<SyncBlockType> for BlockType {
    async fn sync(data_dir: &Path, incoming_blocktypes: SyncBlockType) -> Result<(), Error> {
        // Load existing block types from the given directory.
        let mut blocktypes = BlockType::load(data_dir).await?;

        // Add any incoming block types that don't already exist in the current collection.
        incoming_blocktypes.iter().for_each(|b| {
            if !blocktypes.iter().any(|bt| bt == b) {
                blocktypes.push(b.clone());
            }
        });

        // Save the updated list of block types back to the directory.
        BlockType::save(data_dir, &blocktypes).await
    }
}

// A type to encapsulate data for syncing `TimeBlock` objects.
pub type SyncTimeBlock = HashMap<NaiveDate, Vec<TimeBlock>>; // Maps dates to a list of time blocks.

// Implement the `Sync` trait for `TimeBlock` to handle syncing time blocks.
impl Sync<SyncTimeBlock> for TimeBlock {
    async fn sync(data_dir: &Path, incoming_data: SyncTimeBlock) -> Result<(), Error> {
        // Convert the incoming data into an IndexMap for sorting by date.
        let mut index_map: IndexMap<NaiveDate, Vec<TimeBlock>> = IndexMap::from_iter(incoming_data);
        index_map.sort_unstable_keys();

        // Load the current block information from the directory.
        let self_current_block = CurrentBlock::get(data_dir).await?;

        // Process each date and its associated blocks in the index map.
        for (date, blocks) in index_map {
            // Load the existing time blocks for the current date.
            let self_blocks = TimeBlock::get_day_timeblocks(data_dir, date).await?;

            // Determine the latest end time among the existing time blocks.
            let self_end_time = self_blocks.iter().map(|b| b.end_time).max();

            // If there is an existing end time, handle overlaps and gaps in the time blocks.
            if let Some(self_end_time) = self_end_time {
                // Filter incoming blocks to exclude those starting before the latest end time.
                let blocks = blocks
                    .iter()
                    .filter(|b| b.start_time >= self_end_time)
                    .collect::<Vec<_>>();

                if blocks.is_empty() {
                    // If no new blocks exist, ensure the day ends with a block ending at 23:59:59.
                    if self_end_time.time()
                        != NaiveTime::from_hms_opt(23, 59, 59).ok_or(err_from_type!(
                            ErrorType::Chrono,
                            "Creating end time for {}",
                            date.format("%Y-%m-%d")
                        ))?
                    {
                        let new_block_start_time = self_end_time;
                        let new_block_end_time = new_block_start_time
                            .with_time(NaiveTime::from_hms_opt(23, 59, 59).ok_or(
                                err_from_type!(
                                    ErrorType::Chrono,
                                    "Creating end time for {}",
                                    date.format("%Y-%m-%d")
                                ),
                            )?)
                            .single()
                            .ok_or(err_from_type!(
                                ErrorType::Chrono,
                                "No single time identifiable for end time for {}",
                                date.format("%Y-%m-%d")
                            ))?;

                        // Create and save a new time block for the remaining time in the day.
                        let new_block = TimeBlock::new(
                            new_block_start_time,
                            new_block_end_time,
                            self_current_block.block_type_id,
                            self_current_block.current_block_name.clone(),
                        );
                        if let Err(e) = new_block.save(data_dir).await {
                            println!("Error saving timeblock: {}", e);
                            return Err(e);
                        }
                    }
                } else {
                    // If new blocks exist, handle gaps between the latest end time and the first new block.
                    if self_end_time != blocks[0].start_time {
                        let new_block = TimeBlock::new(
                            self_end_time,
                            blocks[0].start_time,
                            self_current_block.block_type_id,
                            self_current_block.current_block_name.clone(),
                        );
                        if let Err(e) = new_block.save(data_dir).await {
                            println!("Error saving timeblock: {}", e);
                            return Err(e);
                        }
                    }

                    // Save the new blocks to the directory.
                    for block in blocks {
                        if let Err(e) = block.save(data_dir).await {
                            println!("Error saving timeblock: {}", e);
                            return Err(e);
                        }
                    }
                }
            } else {
                // If no existing blocks are found, save all incoming blocks.
                for block in blocks {
                    if let Err(e) = block.save(data_dir).await {
                        println!("Error saving timeblock: {}", e);
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }
}

pub type SyncCurrentBlock = CurrentBlock;

// Implement the `Sync` trait for `CurrentBlock` to handle syncing current blocks.
impl Sync<SyncCurrentBlock> for CurrentBlock {
    async fn sync(data_dir: &Path, incoming_data: SyncCurrentBlock) -> Result<(), Error> {
        incoming_data.save(data_dir).await
    }
}
