use std::time::Duration;

use windows::{
    core::Result,
    Foundation::{
        EventRegistrationToken,
        Numerics::{Vector2, Vector3},
        TypedEventHandler,
    },
    System::{DispatcherQueue, DispatcherQueueTimer},
    UI::{
        Color,
        Composition::{CompositionStretch, Compositor, ContainerVisual, SpriteVisual},
    },
};

use crate::{
    chart::ChartSurface, renderer::Renderer, sources::DataSource, text_block::TextBlock, windows_utils::numerics::ToVector2
};

pub struct App {
    queue: DispatcherQueue,
    renderer: Renderer,
    chart: ChartSurface,
    process_name_text: TextBlock,
    utilization_text: TextBlock,
    chart_visual: SpriteVisual,
    info_root: ContainerVisual,
    data_source: Box<dyn DataSource>,
    timer: DispatcherQueueTimer,
    root: SpriteVisual,
    timer_token: EventRegistrationToken,
}

impl App {
    pub fn new(dpi: u32, data_source: Box<dyn DataSource>) -> Result<Box<Self>> {
        let mut app = Box::new(Self::new_internal(dpi, data_source)?);
        let timer = app.timer.clone();
        let timer_token = timer.Tick(&TypedEventHandler::<_, _>::new({
            // SAFETY: We know that the timer will only tick on the same thread
            // as the dispatcher queue (our UI thread). As long as we remove the tick
            // handler before the end of the lifetime of our perf tracker object,
            // we should be fine.
            let app_ptr = Box::into_raw(app);
            let app_workaround: u64 = app_ptr as _;
            app = unsafe { Box::from_raw(app_ptr) };
            move |_, _| -> Result<()> {
                let app = unsafe { (app_workaround as *mut Self).as_mut().unwrap() };
                app.on_tick()?;
                Ok(())
            }
        }))?;

        app.data_source.start()?;
        app.timer.Start()?;
        app.timer_token = timer_token;

        Ok(app)
    }

    pub fn compositor(&self) -> &Compositor {
        &self.renderer.compositor
    }

    pub fn root(&self) -> &SpriteVisual {
        &self.root
    }

    pub fn shutdown(mut self) -> Result<()> {
        let queue = DispatcherQueue::GetForCurrentThread()?;
        if queue != self.queue {
            panic!("The app must be shutdown on the same thread that created it!");
        }
        // SAFETY: There isn't a race here with the tick handler because we are
        // on the same thread.
        self.timer.RemoveTick(self.timer_token)?;
        self.timer.Stop()?;
        self.data_source.close()?;
        Ok(())
    }

    pub fn on_dpi_changed(&mut self, dpi: u32) -> Result<()> {
        self.chart.set_dpi(&self.renderer, dpi)?;
        self.chart_visual.SetSize(self.chart.size().to_vector2())?;

        self.process_name_text.set_dpi(&self.renderer, dpi)?;
        self.utilization_text.set_dpi(&self.renderer, dpi)?;

        let info_height = {
            let process_name_height = self.process_name_text.root().Size()?;
            let utilization_height = self.utilization_text.root().Size()?;
            process_name_height.Y.max(utilization_height.Y)
        };
        self.info_root.SetSize(Vector2::new(0.0, info_height))?;
        self.info_root
            .SetOffset(Vector3::new(0.0, -info_height, 0.0))?;

        Ok(())
    }

    fn on_tick(&mut self) -> Result<()> {
        let utilization_value = self.data_source.get_current_value()?;
        self.chart.add_point(utilization_value as f32);
        self.chart.set_max_value(self.data_source.gen_current_max()? as f32);
        self.chart.redraw(&self.renderer)?;
        self.utilization_text
            .set_text(&self.renderer, format!("{}{}", utilization_value as i32, self.data_source.unit_label()))?;
        Ok(())
    }

    fn new_internal(dpi: u32, data_source: Box<dyn DataSource>) -> Result<Self> {
        let queue = DispatcherQueue::GetForCurrentThread()?;
        let renderer = Renderer::new()?;

        let compositor = renderer.compositor.clone();
        let root = compositor.CreateSpriteVisual()?;
        root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;
        root.SetBrush(&compositor.CreateColorBrushWithColor(Color {
            A: 255,
            R: 255,
            G: 255,
            B: 255,
        })?)?;

        let max_value = data_source.gen_current_max()?;
        let label = data_source.label()?;
        let unit_label = data_source.unit_label();

        let chart = ChartSurface::new(&renderer, dpi, max_value as f32)?;
        let chart_visual = compositor.CreateSpriteVisual()?;
        chart_visual.SetSize(chart.size().to_vector2())?;
        chart_visual.SetRelativeOffsetAdjustment(Vector3::new(0.5, 0.5, 0.0))?;
        chart_visual.SetAnchorPoint(Vector2::new(0.5, 0.5))?;
        let brush = compositor.CreateSurfaceBrushWithSurface(chart.surface())?;
        brush.SetStretch(CompositionStretch::None)?;
        chart_visual.SetBrush(&brush)?;
        root.Children()?.InsertAtTop(&chart_visual)?;
        chart.redraw(&renderer)?;

        let process_name_text = TextBlock::new(
            &renderer,
            label,
            Color {
                A: 255,
                R: 0,
                G: 0,
                B: 0,
            },
            dpi,
        )?;

        let utilization_text = TextBlock::new(
            &renderer,
            format!("{}{}", 0, unit_label),
            Color {
                A: 255,
                R: 112,
                G: 112,
                B: 112,
            },
            dpi,
        )?;
        let utilization_text_root = utilization_text.root();
        utilization_text_root.SetAnchorPoint(Vector2::new(1.0, 0.0))?;
        utilization_text_root.SetRelativeOffsetAdjustment(Vector3::new(1.0, 0.0, 0.0))?;

        let info_height = {
            let process_name_height = process_name_text.root().Size()?;
            let utilization_height = utilization_text_root.Size()?;
            process_name_height.Y.max(utilization_height.Y)
        };

        let info_root = compositor.CreateContainerVisual()?;
        info_root.SetRelativeSizeAdjustment(Vector2::new(1.0, 0.0))?;
        info_root.SetSize(Vector2::new(0.0, info_height))?;
        info_root.SetOffset(Vector3::new(0.0, -info_height, 0.0))?;
        chart_visual.Children()?.InsertAtTop(&info_root)?;

        let info_root_children = info_root.Children()?;
        info_root_children.InsertAtTop(process_name_text.root())?;
        info_root_children.InsertAtTop(utilization_text_root)?;

        let timer = queue.CreateTimer()?;
        timer.SetInterval(Duration::from_secs(1).into())?;
        timer.SetIsRepeating(true)?;

        Ok(Self {
            queue,
            renderer,
            chart,
            process_name_text,
            utilization_text,
            chart_visual,
            info_root,
            data_source,
            timer,
            root,
            timer_token: Default::default(),
        })
    }
}
