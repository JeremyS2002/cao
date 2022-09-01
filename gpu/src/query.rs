
use ash::vk;

use std::mem::ManuallyDrop as Md;
use std::sync::Arc;
use std::ptr;
use std::time::Duration;

/// A TimeQuery
/// 
/// Used for measuring time taken between commands submitted to command buffers
/// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkQueryPool.html>
/// 
/// Note that although in vulkan time stamp queries are only one type of queries, since they have a different api 
/// they have been separated into a different type to other query types.
pub struct TimeQuery {
    pub(crate) name: Option<String>,
    pub(crate) raw: Md<Arc<vk::QueryPool>>,
    pub(crate) count: u32,
    pub(crate) device: Arc<crate::RawDevice>,
}

impl PartialEq for TimeQuery {
    fn eq(&self, other: &TimeQuery) -> bool {
        **self.raw == **other.raw
    }
}

impl Eq for TimeQuery {}

impl std::hash::Hash for TimeQuery {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self.raw).hash(state)
    }
}

impl Clone for TimeQuery {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            raw: Md::new(Arc::clone(&self.raw)),
            count: self.count,
            device: Arc::clone(&self.device),
        }
    }
}

impl std::fmt::Debug for TimeQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimeQuery id: {:?} name: {:?}", **self.raw, self.name)
    }
}

impl TimeQuery {
    pub fn new(device: &crate::Device, count: u32, name: Option<&str>) -> Result<Self, crate::Error> {
        let create_info = vk::QueryPoolCreateInfo {
            s_type: vk::StructureType::QUERY_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::QueryPoolCreateFlags::empty(),
            query_type: vk::QueryType::TIMESTAMP,
            query_count: count,
            pipeline_statistics: vk::QueryPipelineStatisticFlags::empty(),
        };

        let result = unsafe {
            device.raw.create_query_pool(&create_info, None)
        };

        let raw = match result {
            Ok(p) => p,
            Err(e) => return Err(e.into()),
        };

        let s = Self {
            name: name.as_ref().map(|s| s.to_string()),
            raw: Md::new(Arc::new(raw)),
            count,
            device: Arc::clone(&device.raw),
        };

        if let Some(name) = &name {
            device.raw.set_time_query_name(&s, name.as_ref())?;
        }

        device.raw.check_errors()?;

        Ok(s)
    }

    /// Convert raw time stamps into the durations between events
    /// 
    /// This works assuming one stamp is written between each duration mark.
    /// for example if stamps is vec![0, 1, 2, 3, 4, 5] and timestamp_period=1ns
    /// then the result will be vec![1ns, 1ns, 1ns, 1ns, 1ns]
    pub fn fold_stamps_to_times(&self, stamps: Vec<u64>) -> Vec<Duration> {
        let tick = self.device.limits.timestamp_period as u64;
        let dur = Duration::from_nanos(tick);
        let mut i = stamps.into_iter();
        if let Some(mut start) = i.next() {
            i.fold(Vec::new(), |mut a, end| {
                a.push(dur * (end - start) as u32);
                start = end;
                a
            })
        } else {
            Vec::new()
        }
    }

    /// Convert raw time stamps into durations between events
    /// 
    /// This works assuming that each duration has it's own start and end stamp
    /// for example if stamps is vec![0, 10, 8, 14, 11, 15, 20, 23, 14, 25] and timestamp_period=1ns
    /// then the result will be vec![10ns, 6ns, 3ns, 3ns, 11ns]
    pub fn pair_stamps_to_times(&self, stamps: Vec<u64>) -> Vec<Duration> {
        let tick = self.device.limits.timestamp_period as u64;
        let dur = Duration::from_nanos(tick);
        stamps.chunks(2).map(|a| {
            let s = a[0];
            let e = a[1];
            dur * (e - s) as u32
        }).collect()
    }

    /// Check results of query, if the commands have completed return Vec of timestamps written by cmd_write_timestamp, if not then returns None
    ///
    /// To convert time stamps to times multiply the difference between consecutive timestamps by device.limits.timestamp_period
    pub fn check_results(&self, first_query: u32, query_count: u32) -> Result<Option<Vec<u64>>, crate::Error> {
        assert!(first_query + query_count <= self.count, "Cannot read more queries than the query pool was created with");
        let mut results = vec![0u64; self.count as usize];

        let res = unsafe {
            self.device.get_query_pool_results(
                **self.raw, 
                first_query, 
                query_count, 
                &mut results,
                vk::QueryResultFlags::TYPE_64,
            )
        };

        if res.is_err() {
            if let vk::Result::NOT_READY = res.err().unwrap() {
                return Ok(None)
            }
        }

        match res {
            Ok(_) => {
                Ok(Some(results))
            },
            Err(e) => Err(e.into()),
        }
    }

    /// Check results of query, if the commands have completed return Vec of durations between folded cmd_write_timestamp calls, if not then returns None
    pub fn check_folded_times(&self, first_query: u32, query_count: u32) -> Result<Option<Vec<Duration>>, crate::Error> {
        Ok(self.check_results(first_query, query_count)?.map(|s| self.fold_stamps_to_times(s)))
    }

    /// Check results of query, if the commands have completed return Vec of durations between paired cmd_write_timestamp calls, if not then returns None
    pub fn check_paired_times(&self, first_query: u32, query_count: u32) -> Result<Option<Vec<Duration>>, crate::Error> {
        Ok(self.check_results(first_query, query_count)?.map(|s| self.pair_stamps_to_times(s)))
    }

    /// Get results of query, wait for the commands to complete and return Vec of durations between cmd_write_timestamp calls
    ///
    /// To convert time stamps to times multiply the difference between consecutive timestamps by device.limits.timestamp_period
    pub fn get_results(&self, first_query: u32, query_count: u32) -> Result<Vec<u64>, crate::Error> {
        assert!(first_query + query_count <= self.count, "Cannot read more queries than the query pool was created with");
        let mut results = vec![0u64; self.count as usize];

        let res = unsafe {
            self.device.get_query_pool_results(
                **self.raw, 
                first_query, 
                query_count, 
                &mut results,
                vk::QueryResultFlags::TYPE_64
                    | vk::QueryResultFlags::WAIT,
            )
        };

        match res {
            Ok(_) => Ok(results),
            Err(e) => Err(e.into()),
        }
    }

    /// Get results of query, wait for the commands to complete and return Vec of folded duraions between cmd_write_timestamp calls
    pub fn get_folded_times(&self, first_query: u32, query_count: u32) -> Result<Vec<Duration>, crate::Error> {
        Ok(self.fold_stamps_to_times(self.get_results(first_query, query_count)?))
    }

    /// Get results of query, wait for the commands to complete and return Vec of paired duraions between cmd_write_timestamp calls
    pub fn get_paired_times(&self, first_query: u32, query_count: u32) -> Result<Vec<Duration>, crate::Error> {
        Ok(self.pair_stamps_to_times(self.get_results(first_query, query_count)?))
    }
}

impl Drop for TimeQuery {
    fn drop(&mut self) {
        unsafe {
            let raw = Md::take(&mut self.raw);
            if let Ok(raw) = Arc::try_unwrap(raw) {
                self.device.destroy_query_pool(raw, None);
            }
        }
    }
}

// pub struct QueryDesc {
//     pub ty: crate::QueryType,
//     pub count: u32,
//     pub name: Option<String>,
// }

// pub struct Query {
//     pub(crate) name: Option<String>,
//     pub(crate) raw: Md<Arc<vk::QueryPool>>,
//     pub(crate) device: Arc<crate::RawDevice>,
// }

// impl PartialEq for Query {
//     fn eq(&self, other: &Query) -> bool {
//         **self.raw == **other.raw
//     }
// }

// impl Eq for Query {}

// impl std::hash::Hash for Query {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         (**self.raw).hash(state)
//     }
// }

// impl Clone for Query {
//     fn clone(&self) -> Self {
//         Self {
//             name: self.name.clone(),
//             raw: Md::new(Arc::clone(&self.raw)),
//             device: Arc::clone(&self.device),
//         }
//     }
// }

// impl std::fmt::Debug for Query {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Query id: {:?} name: {:?}", **self.raw, self.name)
//     }
// }

// impl Query {
//     pub fn get_results(&self) -> Result<(), crate::Error> {
//         let mut data = Vec::<u8>::new();
//         let res = unsafe {
//             self.device.get_query_pool_results(
//                 **self.raw, 
//                 0, 
//                 1, 
//                 &mut data, 
//                 vk::QueryResultFlags::TYPE_64
//             )
//         };
        
//         todo!();
//     }
// }