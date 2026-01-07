from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'components/anchor/index.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_view_object = resolve('view_object')
    l_0_sdoc_entity = resolve('sdoc_entity')
    l_0_incoming_links = missing
    try:
        t_1 = environment.filters['length']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No filter named 'length' found.")
    try:
        t_2 = environment.tests['none']
    except KeyError:
        @internalcode
        def t_2(*unused):
            raise TemplateRuntimeError("No test named 'none' found.")
    pass
    yield '<sdoc-anchor\n  id="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_local_anchor'), (undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity)))
    yield '"\n  node-role="'
    yield escape(context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_type_string')))
    yield '"\n  '
    if (not t_2(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_uid'))):
        pass
        yield 'data-uid="'
        yield escape(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_uid'))
        yield '"'
    yield '>'
    l_0_incoming_links = context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'traceability_index'), 'get_incoming_links'), (undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity))
    context.vars['incoming_links'] = l_0_incoming_links
    context.exported_vars.add('incoming_links')
    if ((not t_2((undefined(name='incoming_links') if l_0_incoming_links is missing else l_0_incoming_links))) and (t_1((undefined(name='incoming_links') if l_0_incoming_links is missing else l_0_incoming_links)) > 0)):
        pass
        yield '<template>\n  Incoming link'
        if (t_1((undefined(name='incoming_links') if l_0_incoming_links is missing else l_0_incoming_links)) > 1):
            pass
            yield 's'
        yield ' from:\n  '
        for l_1_incoming_link in (undefined(name='incoming_links') if l_0_incoming_links is missing else l_0_incoming_links):
            l_1_incoming_link_parent_node = l_1_incoming_link_href = missing
            _loop_vars = {}
            pass
            l_1_incoming_link_parent_node = context.call(environment.getattr(l_1_incoming_link, 'parent_node'), _loop_vars=_loop_vars)
            _loop_vars['incoming_link_parent_node'] = l_1_incoming_link_parent_node
            l_1_incoming_link_href = context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_node_link'), context.call(environment.getattr(l_1_incoming_link, 'parent_node'), _loop_vars=_loop_vars), _loop_vars=_loop_vars)
            _loop_vars['incoming_link_href'] = l_1_incoming_link_href
            yield '<a href="'
            yield escape((undefined(name='incoming_link_href') if l_1_incoming_link_href is missing else l_1_incoming_link_href))
            yield '">\n    '
            yield escape(context.call(environment.getattr((undefined(name='incoming_link_parent_node') if l_1_incoming_link_parent_node is missing else l_1_incoming_link_parent_node), 'get_display_title'), _loop_vars=_loop_vars))
            yield '\n  </a>\n  '
        l_1_incoming_link = l_1_incoming_link_parent_node = l_1_incoming_link_href = missing
        yield '</template>'
    yield '</sdoc-anchor>'

blocks = {}
debug_info = '2=27&3=29&4=31&5=34&8=37&9=40&11=43&12=47&13=51&14=53&15=56&16=58'