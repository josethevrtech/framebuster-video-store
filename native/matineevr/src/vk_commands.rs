use super::{renderer::Completed, vk_device::Device};
use crate::statistics::{Statistics, timed};
use anyhow::Result;
use ash::vk;
use std::rc::Rc;

pub struct Commands {
    pub command: vk::CommandBuffer,
    pub frame: Option<Rc<crate::media::Frame>>,
    fence: vk::Fence,
    pool: vk::CommandPool,
    queries: vk::QueryPool,
    sequence: Option<usize>,
    device: Rc<Device>,
}

impl Commands {
    pub fn new(device: Rc<Device>) -> Result<Self> {
        let mut result = Self {
            command: vk::CommandBuffer::null(),
            frame: None,
            fence: vk::Fence::null(),
            pool: vk::CommandPool::null(),
            queries: vk::QueryPool::null(),
            sequence: None,
            device,
        };
        unsafe {
            let d = &result.device.api;
            result.pool = d.create_command_pool(
                &vk::CommandPoolCreateInfo::default().queue_family_index(result.device.family),
                None,
            )?;
            result.command = d.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(result.pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1),
            )?[0];
            result.fence = d.create_fence(&vk::FenceCreateInfo::default(), None)?;
            result.queries = d.create_query_pool(
                &vk::QueryPoolCreateInfo::default()
                    .query_type(vk::QueryType::TIMESTAMP)
                    .query_count(3),
                None,
            )?;
        }
        Ok(result)
    }

    pub fn collect(&mut self) -> Result<Option<Completed>> {
        self.collect_stats(None)
    }

    pub fn collect_stats(
        &mut self,
        mut stats: Option<&mut Statistics>,
    ) -> Result<Option<Completed>> {
        let Some(sequence) = self.sequence else {
            return Ok(None);
        };
        unsafe {
            let d = &self.device.api;
            timed(stats.as_deref_mut(), "gpu_fence_wait_ms", || {
                d.wait_for_fences(&[self.fence], true, 10_000_000_000)
            })?;
            let mut stamps = [0_u64; 3];
            d.get_query_pool_results(self.queries, 0, &mut stamps, vk::QueryResultFlags::TYPE_64)?;
            self.sequence = None;
            self.frame = None;
            let elapsed = |a: u64, b: u64| {
                (b.wrapping_sub(a) & self.device.timestamp_mask) as f64
                    * self.device.timestamp_period
                    * 1e-6
            };
            if let Some(stats) = stats {
                stats.sample("gpu_draw_ms", elapsed(stamps[1], stamps[2]));
            }
            Ok(Some((
                sequence,
                [elapsed(stamps[0], stamps[1]), elapsed(stamps[1], stamps[2])],
            )))
        }
    }

    pub fn begin(&self) -> Result<()> {
        unsafe {
            let d = &self.device.api;
            d.reset_command_pool(self.pool, vk::CommandPoolResetFlags::empty())?;
            d.begin_command_buffer(
                self.command,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;
            d.cmd_reset_query_pool(self.command, self.queries, 0, 3);
        }
        self.timestamp(0);
        Ok(())
    }

    pub fn timestamp(&self, index: u32) {
        unsafe {
            self.device.api.cmd_write_timestamp(
                self.command,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                self.queries,
                index,
            );
        }
    }

    pub fn submit(&mut self, sequence: usize) -> Result<()> {
        unsafe {
            let d = &self.device.api;
            d.end_command_buffer(self.command)?;
            d.reset_fences(&[self.fence])?;
            d.queue_submit(
                self.device.queue,
                &[vk::SubmitInfo::default().command_buffers(&[self.command])],
                self.fence,
            )?;
        }
        self.sequence = Some(sequence);
        Ok(())
    }
}

impl Drop for Commands {
    fn drop(&mut self) {
        unsafe {
            let d = &self.device.api;
            if self.sequence.is_some()
                && let Err(error) = d.device_wait_idle()
            {
                eprintln!("Vulkan command shutdown: {error}");
            }
            d.destroy_query_pool(self.queries, None);
            d.destroy_fence(self.fence, None);
            d.destroy_command_pool(self.pool, None);
        }
    }
}
