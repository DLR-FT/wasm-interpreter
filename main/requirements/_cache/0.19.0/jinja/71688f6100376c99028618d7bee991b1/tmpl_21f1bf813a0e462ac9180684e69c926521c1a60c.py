from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'components/node_field/links/index.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_view_object = resolve('view_object')
    l_0_sdoc_entity = resolve('sdoc_entity')
    try:
        t_1 = environment.tests['none']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No test named 'none' found.")
    pass
    if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'traceability_index'), 'has_parent_requirements'), (undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity)):
        pass
        yield '\n    <sdoc-node-field-label>RELATIONS (Parent):</sdoc-node-field-label>\n    <sdoc-node-field data-field-label="parent relations">\n      <ul class="requirement__link">'
        for (l_1_sdoc_entity, l_1_relation_role_) in context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'traceability_index'), 'get_parent_relations_with_roles'), (undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity)):
            _loop_vars = {}
            pass
            yield '\n        <li>\n          <a data-turbo="false" class="requirement__link-parent" href="'
            yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_node_link'), l_1_sdoc_entity, _loop_vars=_loop_vars))
            yield '">'
            if environment.getattr(l_1_sdoc_entity, 'reserved_uid'):
                pass
                yield '\n            <span class="requirement__parent-uid">'
                yield escape(environment.getattr(l_1_sdoc_entity, 'reserved_uid'))
                yield '</span>'
            yield '\n            '
            yield escape((environment.getattr(l_1_sdoc_entity, 'reserved_title') if environment.getattr(l_1_sdoc_entity, 'reserved_title') else ''))
            yield '\n            '
            if (not t_1(l_1_relation_role_)):
                pass
                yield '\n              <span class="requirement__type-tag">('
                yield escape(l_1_relation_role_)
                yield ')</span>\n            '
            yield '\n          </a>\n        </li>'
        l_1_sdoc_entity = l_1_relation_role_ = missing
        yield '\n      </ul>\n    </sdoc-node-field>'
    if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'traceability_index'), 'has_children_requirements'), (undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity)):
        pass
        yield '\n    <sdoc-node-field-label>RELATIONS (Child):</sdoc-node-field-label>\n    <sdoc-node-field data-field-label="child relations">\n      <ul class="requirement__link">'
        for (l_1_sdoc_entity, l_1_relation_role_) in context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'traceability_index'), 'get_child_relations_with_roles'), (undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity)):
            _loop_vars = {}
            pass
            yield '\n        <li>\n          <a data-turbo="false" class="requirement__link-child" href="'
            yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_node_link'), l_1_sdoc_entity, _loop_vars=_loop_vars))
            yield '">'
            if environment.getattr(l_1_sdoc_entity, 'reserved_uid'):
                pass
                yield '\n            <span class="requirement__child-uid">'
                yield escape(environment.getattr(l_1_sdoc_entity, 'reserved_uid'))
                yield '</span>'
            yield '\n            '
            yield escape((environment.getattr(l_1_sdoc_entity, 'reserved_title') if environment.getattr(l_1_sdoc_entity, 'reserved_title') else ''))
            yield '\n            '
            if (not t_1(l_1_relation_role_)):
                pass
                yield '\n              <span class="requirement__type-tag">('
                yield escape(l_1_relation_role_)
                yield ')</span>\n            '
            yield '\n          </a>\n        </li>'
        l_1_sdoc_entity = l_1_relation_role_ = missing
        yield '\n      </ul>\n    </sdoc-node-field>'

blocks = {}
debug_info = '2=19&6=22&8=26&9=28&10=31&12=34&13=36&14=39&23=44&27=47&29=51&30=53&31=56&33=59&34=61&35=64'