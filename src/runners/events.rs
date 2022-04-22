use zk_evm::{ethereum_types::Address, testing::event_sink::EventMessage};

#[derive(Clone)]
pub struct SolidityLikeEvent {
    pub shard_id: u8,
    pub tx_number_in_block: u16,
    pub address: Address,
    pub topics: Vec<[u8; 32]>,
    pub data: Vec<u8>,
}

impl std::fmt::Debug for SolidityLikeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolidityLikeEvent")
            .field("shard_id", &self.shard_id)
            .field("tx_number_in_block", &self.tx_number_in_block)
            .field("address", &self.address)
            .field(
                "topics",
                &format_args!(
                    "[{}]",
                    self.topics
                        .iter()
                        .map(|el| format!("0x{}", hex::encode(&el)))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            )
            .field("value", &format_args!("0x{}", hex::encode(&self.data)))
            .finish()
    }
}

pub fn merge_events(events: Vec<EventMessage>) -> Vec<SolidityLikeEvent> {
    let mut result = vec![];
    let mut current: Option<(usize, u32, SolidityLikeEvent)> = None;

    for message in events.into_iter() {
        if !message.is_first {
            let EventMessage {
                shard_id,
                is_first: _,
                tx_number_in_block,
                address,
                key,
                value,
            } = message;

            if let Some((mut remaining_data_length, mut remaining_topics, mut event)) =
                current.take()
            {
                if event.address != address
                    || event.shard_id != shard_id
                    || event.tx_number_in_block != tx_number_in_block
                {
                    continue;
                }
                let mut data_0 = [0u8; 32];
                let mut data_1 = [0u8; 32];
                key.to_big_endian(&mut data_0);
                value.to_big_endian(&mut data_1);
                for el in [data_0, data_1].into_iter() {
                    if remaining_topics != 0 {
                        event.topics.push(el);
                        remaining_topics -= 1;
                    } else if remaining_data_length != 0 {
                        if remaining_data_length >= 32 {
                            event.data.extend_from_slice(&el);
                            remaining_data_length -= 32;
                        } else {
                            event.data.extend_from_slice(&el[..remaining_data_length]);
                            remaining_data_length = 0;
                        }
                    }
                }

                if remaining_data_length != 0 || remaining_topics != 0 {
                    current = Some((remaining_data_length, remaining_topics, event))
                } else {
                    result.push(event);
                }
            }
        } else {
            // start new one. First take the old one only if it's well formed
            if let Some((remaining_data_length, remaining_topics, event)) = current.take() {
                if remaining_data_length == 0 && remaining_topics == 0 {
                    result.push(event);
                }
            }

            let EventMessage {
                shard_id,
                is_first: _,
                tx_number_in_block,
                address,
                key,
                value,
            } = message;
            // split key as our internal marker. Ignore higher bits
            let mut num_topics = key.0[0] as u32;
            let data_length = (key.0[0] >> 32) as usize;
            let mut buffer = [0u8; 32];
            value.to_big_endian(&mut buffer);

            let topics = if num_topics == 0 {
                vec![]
            } else {
                num_topics -= 1;
                vec![buffer]
            };

            let new_event = SolidityLikeEvent {
                shard_id,
                tx_number_in_block,
                address,
                topics,
                data: vec![],
            };

            current = Some((data_length, num_topics, new_event))
        }
    }

    // add the last one
    if let Some((remaining_data_length, remaining_topics, event)) = current.take() {
        if remaining_data_length == 0 && remaining_topics == 0 {
            result.push(event);
        }
    }

    result
}

// This is just a copy-paste from compiler-tester repo that allows to pass
// data in a serialized forms without much of the interpretation

use serde::{Deserialize, Serialize};

///
/// The compiler test case outcome event.
///
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Event {
    /// The indexed topics.
    pub topics: Vec<String>,
    /// The ordinary values.
    pub values: Vec<String>,
}

impl Into<Event> for SolidityLikeEvent {
    fn into(self) -> Event {
        // topics are formatted as hex with 0x
        let topics = self
            .topics
            .into_iter()
            .map(|el| format!("0x{}", hex::encode(&el)))
            .collect();
        // values are formatted as integers

        let mut values = vec![];
        let mut it = self.data.chunks_exact(32);
        use crate::U256;
        for el in &mut it {
            let as_integer = U256::from_big_endian(el);
            values.push(format!("{}", as_integer));
        }

        let remainder = it.remainder();
        if !remainder.is_empty() {
            let as_integer = U256::from_big_endian(remainder);
            values.push(format!("{}", as_integer));
        }

        Event { topics, values }
    }
}
