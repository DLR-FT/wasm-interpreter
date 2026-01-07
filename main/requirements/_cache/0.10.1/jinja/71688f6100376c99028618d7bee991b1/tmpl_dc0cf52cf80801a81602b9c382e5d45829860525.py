from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/project_index/project_map_section.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_section = resolve('section')
    try:
        t_1 = environment.tests['defined']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No test named 'defined' found.")
    pass
    def macro():
        t_2 = []
        pass
        return concat(t_2)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, t_1((undefined(name='section') if l_0_section is missing else l_0_section)), None, caller=caller)
    l_1_node = (undefined(name='section') if l_0_section is missing else l_0_section)
    pass
    template = environment.get_template('screens/project_index/project_map_node.jinja', 'screens/project_index/project_map_section.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {'node': l_1_node}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    l_1_node = missing
    for l_1_node_ in environment.getattr((undefined(name='section') if l_0_section is missing else l_0_section), 'section_contents'):
        _loop_vars = {}
        pass
        if ((environment.getattr(environment.getattr(l_1_node_, '__class__'), '__name__') == 'DocumentFromFile') or environment.getattr(l_1_node_, 'is_section')):
            pass
            l_2_section = l_1_node_
            pass
            template = environment.get_template('screens/project_index/project_map_section.jinja', 'screens/project_index/project_map_section.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'section': l_2_section, 'node_': l_1_node_}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            l_2_section = missing
        else:
            pass
            l_2_node = l_1_node_
            pass
            template = environment.get_template('screens/project_index/project_map_node.jinja', 'screens/project_index/project_map_section.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'node': l_2_node, 'node_': l_1_node_}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            l_2_node = missing
    l_1_node_ = missing

blocks = {}
debug_info = '1=18&3=26&6=33&7=36&9=40&13=51'