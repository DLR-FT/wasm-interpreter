from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'components/node/index.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_sdoc_entity = resolve('sdoc_entity')
    l_0_view_object = resolve('view_object')
    l_0_node_type = l_0_is_not_standalone = l_0_is_included = l_0_node_type_string = l_0_copy_to_clipboard = missing
    try:
        t_1 = environment.tests['none']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No test named 'none' found.")
    pass
    yield '\n\n\n\n'
    l_0_node_type = context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_type_string'))
    context.vars['node_type'] = l_0_node_type
    context.exported_vars.add('node_type')
    yield '\n'
    l_0_is_not_standalone = (environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_running_on_server') and (not environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'standalone')))
    context.vars['is_not_standalone'] = l_0_is_not_standalone
    context.exported_vars.add('is_not_standalone')
    yield '\n'
    l_0_is_included = context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'document_is_included'))
    context.vars['is_included'] = l_0_is_included
    context.exported_vars.add('is_included')
    yield '\n'
    l_0_node_type_string = context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_node_type_string'))
    context.vars['node_type_string'] = l_0_node_type_string
    context.exported_vars.add('node_type_string')
    yield '\n\n<turbo-frame'
    if (undefined(name='is_not_standalone') if l_0_is_not_standalone is missing else l_0_is_not_standalone):
        pass
        yield '\n  id="article-'
        yield escape(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_mid'))
        yield '"\n'
    yield '>\n\n  <sdoc-node'
    if (undefined(name='is_not_standalone') if l_0_is_not_standalone is missing else l_0_is_not_standalone):
        pass
        yield '\n      data-editable_node="on"\n      data-controller="anchor_controller"'
    yield '\n    node-role="'
    yield escape((undefined(name='node_type') if l_0_node_type is missing else l_0_node_type))
    yield '"\n    '
    if context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'is_requirement')):
        pass
        yield '\n      node-view="'
        yield escape(context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_requirement_style_mode')))
        yield '"'
    if (not t_1((undefined(name='node_type_string') if l_0_node_type_string is missing else l_0_node_type_string))):
        pass
        yield '\n      show-node-type-name="'
        yield escape((undefined(name='node_type_string') if l_0_node_type_string is missing else l_0_node_type_string))
        yield '"'
    yield '\n    data-included-document="'
    yield escape((undefined(name='is_included') if l_0_is_included is missing else l_0_is_included))
    yield '"\n    data-testid="node-'
    yield escape((undefined(name='node_type') if l_0_node_type is missing else l_0_node_type))
    yield '"\n  >\n\n    '
    if context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'should_display_stable_link'), (undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity)):
        pass
        yield '\n      '
        l_1_path = context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'get_stable_link'), (undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity))
        pass
        yield '\n        '
        template = environment.get_template('components/node/copy_stable_link_button.jinja', 'components/node/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'path': l_1_path, 'copy_to_clipboard': l_0_copy_to_clipboard, 'is_included': l_0_is_included, 'is_not_standalone': l_0_is_not_standalone, 'node_type': l_0_node_type, 'node_type_string': l_0_node_type_string}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n      '
        l_1_path = missing
    yield '\n    '
    template = environment.get_template('components/anchor/index.jinja', 'components/node/index.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {'copy_to_clipboard': l_0_copy_to_clipboard, 'is_included': l_0_is_included, 'is_not_standalone': l_0_is_not_standalone, 'node_type': l_0_node_type, 'node_type_string': l_0_node_type_string}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '\n\n    '
    l_0_copy_to_clipboard = True
    context.vars['copy_to_clipboard'] = l_0_copy_to_clipboard
    context.exported_vars.add('copy_to_clipboard')
    yield '\n    '
    yield from context.blocks['sdoc_entity'][0](context)
    yield '\n\n    '
    if (undefined(name='is_not_standalone') if l_0_is_not_standalone is missing else l_0_is_not_standalone):
        pass
        template = environment.get_template('components/node/node_controls/index.jinja', 'components/node/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'copy_to_clipboard': l_0_copy_to_clipboard, 'is_included': l_0_is_included, 'is_not_standalone': l_0_is_not_standalone, 'node_type': l_0_node_type, 'node_type_string': l_0_node_type_string}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
    yield '</sdoc-node>\n</turbo-frame>'

def block_sdoc_entity(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    pass
    yield '\n      \n      '
    def macro():
        t_2 = []
        pass
        return concat(t_2)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, 0, 'Must not reach here!', caller=caller, _block_vars=_block_vars)
    yield '\n    '

blocks = {'sdoc_entity': block_sdoc_entity}
debug_info = '15=21&16=25&17=29&18=33&21=37&22=40&27=43&31=47&32=49&33=52&35=54&36=57&38=60&39=62&42=64&44=70&49=79&56=86&59=90&65=92&66=94&59=102&61=111'