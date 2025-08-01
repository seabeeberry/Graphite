use super::shape_utility::ShapeToolModifierKey;
use super::*;
use crate::messages::portfolio::document::graph_operation::utility_types::TransformIn;
use crate::messages::portfolio::document::node_graph::document_node_definitions::resolve_document_node_type;
use crate::messages::portfolio::document::utility_types::document_metadata::LayerNodeIdentifier;
use crate::messages::portfolio::document::utility_types::network_interface::{InputConnector, NodeTemplate};
use crate::messages::tool::common_functionality::gizmos::shape_gizmos::sweep_angle_gizmo::{SweepAngleGizmo, SweepAngleGizmoState};
use crate::messages::tool::common_functionality::graph_modification_utils;
use crate::messages::tool::common_functionality::shapes::shape_utility::{ShapeGizmoHandler, arc_outline};
use crate::messages::tool::tool_messages::tool_prelude::*;
use glam::DAffine2;
use graph_craft::document::NodeInput;
use graph_craft::document::value::TaggedValue;
use graphene_std::vector::misc::ArcType;
use std::collections::VecDeque;

#[derive(Clone, Debug, Default)]
pub struct ArcGizmoHandler {
	sweep_angle_gizmo: SweepAngleGizmo,
}

impl ArcGizmoHandler {
	pub fn new() -> Self {
		Self { ..Default::default() }
	}
}

impl ShapeGizmoHandler for ArcGizmoHandler {
	fn handle_state(&mut self, selected_shape_layers: LayerNodeIdentifier, mouse_position: DVec2, document: &DocumentMessageHandler, _responses: &mut VecDeque<Message>) {
		self.sweep_angle_gizmo.handle_actions(selected_shape_layers, document, mouse_position);
	}

	fn is_any_gizmo_hovered(&self) -> bool {
		self.sweep_angle_gizmo.hovered()
	}

	fn handle_click(&mut self) {
		if self.sweep_angle_gizmo.hovered() {
			self.sweep_angle_gizmo.update_state(SweepAngleGizmoState::Dragging);
		}
	}

	fn handle_update(&mut self, _drag_start: DVec2, document: &DocumentMessageHandler, input: &InputPreprocessorMessageHandler, responses: &mut VecDeque<Message>) {
		if self.sweep_angle_gizmo.is_dragging_or_snapped() {
			self.sweep_angle_gizmo.update_arc(document, input, responses);
		}
	}

	fn dragging_overlays(
		&self,
		document: &DocumentMessageHandler,
		input: &InputPreprocessorMessageHandler,
		_shape_editor: &mut &mut crate::messages::tool::common_functionality::shape_editor::ShapeState,
		mouse_position: DVec2,
		overlay_context: &mut crate::messages::portfolio::document::overlays::utility_types::OverlayContext,
	) {
		if self.sweep_angle_gizmo.is_dragging_or_snapped() {
			self.sweep_angle_gizmo.overlays(None, document, input, mouse_position, overlay_context);
			arc_outline(self.sweep_angle_gizmo.layer, document, overlay_context);
		}
	}

	fn overlays(
		&self,
		document: &DocumentMessageHandler,
		selected_shape_layers: Option<LayerNodeIdentifier>,
		input: &InputPreprocessorMessageHandler,
		_shape_editor: &mut &mut crate::messages::tool::common_functionality::shape_editor::ShapeState,
		mouse_position: DVec2,
		overlay_context: &mut crate::messages::portfolio::document::overlays::utility_types::OverlayContext,
	) {
		self.sweep_angle_gizmo.overlays(selected_shape_layers, document, input, mouse_position, overlay_context);

		arc_outline(selected_shape_layers.or(self.sweep_angle_gizmo.layer), document, overlay_context);
	}

	fn mouse_cursor_icon(&self) -> Option<MouseCursorIcon> {
		if self.sweep_angle_gizmo.hovered() || self.sweep_angle_gizmo.is_dragging_or_snapped() {
			return Some(MouseCursorIcon::Default);
		}

		None
	}

	fn cleanup(&mut self) {
		self.sweep_angle_gizmo.cleanup();
	}
}
#[derive(Default)]
pub struct Arc;

impl Arc {
	pub fn create_node(arc_type: ArcType) -> NodeTemplate {
		let node_type = resolve_document_node_type("Arc").expect("Ellipse node does not exist");
		node_type.node_template_input_override([
			None,
			Some(NodeInput::value(TaggedValue::F64(0.5), false)),
			Some(NodeInput::value(TaggedValue::F64(0.), false)),
			Some(NodeInput::value(TaggedValue::F64(270.), false)),
			Some(NodeInput::value(TaggedValue::ArcType(arc_type), false)),
		])
	}

	pub fn update_shape(
		document: &DocumentMessageHandler,
		ipp: &InputPreprocessorMessageHandler,
		layer: LayerNodeIdentifier,
		shape_tool_data: &mut ShapeToolData,
		modifier: ShapeToolModifierKey,
		responses: &mut VecDeque<Message>,
	) {
		let (center, lock_ratio) = (modifier[0], modifier[1]);
		if let Some([start, end]) = shape_tool_data.data.calculate_points(document, ipp, center, lock_ratio) {
			let Some(node_id) = graph_modification_utils::get_arc_id(layer, &document.network_interface) else {
				return;
			};

			let dimensions = (start - end).abs();
			let mut scale = DVec2::ONE;
			let radius: f64;

			// We keep the smaller dimension's scale at 1 and scale the other dimension accordingly
			if dimensions.x > dimensions.y {
				scale.x = dimensions.x / dimensions.y;
				scale.y = 1.;
				radius = dimensions.y / 2.;
			} else {
				scale.y = dimensions.y / dimensions.x;
				scale.x = 1.;
				radius = dimensions.x / 2.;
			}

			responses.add(NodeGraphMessage::SetInput {
				input_connector: InputConnector::node(node_id, 1),
				input: NodeInput::value(TaggedValue::F64(radius), false),
			});

			responses.add(GraphOperationMessage::TransformSet {
				layer,
				transform: DAffine2::from_scale_angle_translation(scale, 0., start.midpoint(end)),
				transform_in: TransformIn::Viewport,
				skip_rerender: false,
			});
		}
	}
}
