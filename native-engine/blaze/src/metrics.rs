use std::sync::Arc;

use datafusion::physical_plan::ExecutionPlan;
use jni::objects::JObject;

use datafusion_ext::jni_call;
use datafusion_ext::jni_new_string;

const REPORTED_METRICS: &[&str] = &[
    "input_rows",
    "input_batches",
    "output_rows",
    "output_batches",
    "elapsed_compute",
    "join_time",
];

pub fn update_spark_metric_node(
    metric_node: JObject,
    execution_plan: Arc<dyn ExecutionPlan>,
) -> datafusion::error::Result<()> {
    // update current node
    update_metrics(
        metric_node,
        &execution_plan
            .metrics()
            .unwrap_or_default()
            .iter()
            .map(|m| m.value())
            .map(|m| (m.name(), m.as_usize() as i64))
            .collect::<Vec<_>>(),
    )?;

    // update children nodes
    for (i, child_plan) in execution_plan.children().iter().enumerate() {
        let child_metric_node = jni_call!(
            SparkMetricNode(metric_node).getChild(i as i32) -> JObject
        )?;
        update_spark_metric_node(child_metric_node, child_plan.clone())?;
    }
    Ok(())
}

fn update_metrics(
    metric_node: JObject,
    metric_values: &[(&str, i64)],
) -> datafusion::error::Result<()> {
    for &(name, value) in metric_values {
        if REPORTED_METRICS.contains(&name) {
            let jname = jni_new_string!(&name)?;
            jni_call!(SparkMetricNode(metric_node).add(jname, value) -> ())?;
        }
    }
    Ok(())
}
