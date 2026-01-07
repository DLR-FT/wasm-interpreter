from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'components/node_field/title/index.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_sdoc_entity = resolve('sdoc_entity')
    l_0_node_view = resolve('node_view')
    l_0_title_has_h_level = resolve('title_has_h_level')
    l_0_h_level = resolve('h_level')
    l_0_field_content_ = resolve('field_content_')
    l_0_title_number = resolve('title_number')
    l_0_title = resolve('title')
    try:
        t_1 = environment.filters['safe']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No filter named 'safe' found.")
    try:
        t_2 = environment.tests['none']
    except KeyError:
        @internalcode
        def t_2(*unused):
            raise TemplateRuntimeError("No test named 'none' found.")
    try:
        t_3 = environment.tests['true']
    except KeyError:
        @internalcode
        def t_3(*unused):
            raise TemplateRuntimeError("No test named 'true' found.")
    pass
    if environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_title'):
        pass
        yield '\n  <sdoc-node-title\n    data-level="'
        yield escape(environment.getattr(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'context'), 'title_number_string'))
        yield '"\n  >'
        l_0_node_view = context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_requirement_style_mode'))
        context.vars['node_view'] = l_0_node_view
        context.exported_vars.add('node_view')
        yield '\n    '
        l_0_title_has_h_level = (True if (environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'is_composite') and ((undefined(name='node_view') if l_0_node_view is missing else l_0_node_view) not in ['table', 'zebra', 'simple', 'inline'])) else cond_expr_undefined("the inline if-expression on line 10 in 'components/node_field/title/index.jinja' evaluated to false and no else section was defined."))
        context.vars['title_has_h_level'] = l_0_title_has_h_level
        context.exported_vars.add('title_has_h_level')
        l_0_h_level = ((environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'ng_level') + 1) if (environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'ng_level') < 6) else 6)
        context.vars['h_level'] = l_0_h_level
        context.exported_vars.add('h_level')
        if (undefined(name='title_has_h_level') if l_0_title_has_h_level is missing else l_0_title_has_h_level):
            pass
            yield '<h'
            yield escape((undefined(name='h_level') if l_0_h_level is missing else l_0_h_level))
            yield '>'
        l_0_field_content_ = ''
        context.vars['field_content_'] = l_0_field_content_
        context.exported_vars.add('field_content_')
        if t_3((undefined(name='title_number') if l_0_title_number is missing else l_0_title_number)):
            pass
            if environment.getattr(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'context'), 'title_number_string'):
                pass
                l_0_field_content_ = (((undefined(name='field_content_') if l_0_field_content_ is missing else l_0_field_content_) + environment.getattr(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'context'), 'title_number_string')) + Markup('.&nbsp;'))
                context.vars['field_content_'] = l_0_field_content_
                context.exported_vars.add('field_content_')
        l_0_title = environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_title')
        context.vars['title'] = l_0_title
        context.exported_vars.add('title')
        if (not t_2((undefined(name='title') if l_0_title is missing else l_0_title))):
            pass
            l_0_field_content_ = ((undefined(name='field_content_') if l_0_field_content_ is missing else l_0_field_content_) + (undefined(name='title') if l_0_title is missing else l_0_title))
            context.vars['field_content_'] = l_0_field_content_
            context.exported_vars.add('field_content_')
        l_1_field_content = (undefined(name='field_content_') if l_0_field_content_ is missing else l_0_field_content_)
        pass
        template = environment.get_template('components/field/index.jinja', 'components/node_field/title/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'field_content': l_1_field_content, 'field_content_': l_0_field_content_, 'h_level': l_0_h_level, 'node_view': l_0_node_view, 'title': l_0_title, 'title_has_h_level': l_0_title_has_h_level}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        l_1_field_content = missing
        if (undefined(name='title_has_h_level') if l_0_title_has_h_level is missing else l_0_title_has_h_level):
            pass
            yield '</h'
            yield escape((undefined(name='h_level') if l_0_h_level is missing else l_0_h_level))
            yield '>'
        yield '\n  </sdoc-node-title>'

blocks = {}
debug_info = '2=36&4=39&7=41&10=45&15=48&19=51&22=56&24=59&25=61&27=63&31=66&32=69&34=71&39=76&42=83'