from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'rst/anchor.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_link_renderer = resolve('link_renderer')
    l_0_anchor = resolve('anchor')
    l_0_traceability_index = resolve('traceability_index')
    l_0_local_anchor = l_0_incoming_links = missing
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
    yield '\n\n.. raw:: html\n\n    '
    l_0_local_anchor = context.call(environment.getattr((undefined(name='link_renderer') if l_0_link_renderer is missing else l_0_link_renderer), 'render_local_anchor'), (undefined(name='anchor') if l_0_anchor is missing else l_0_anchor))
    context.vars['local_anchor'] = l_0_local_anchor
    context.exported_vars.add('local_anchor')
    yield '<sdoc-anchor id="'
    yield escape((undefined(name='local_anchor') if l_0_local_anchor is missing else l_0_local_anchor))
    yield '" node-role="section" data-uid="'
    yield escape((undefined(name='local_anchor') if l_0_local_anchor is missing else l_0_local_anchor))
    yield '" data-anchor="'
    yield escape((undefined(name='local_anchor') if l_0_local_anchor is missing else l_0_local_anchor))
    yield '" style="top:unset">'
    l_0_incoming_links = context.call(environment.getattr((undefined(name='traceability_index') if l_0_traceability_index is missing else l_0_traceability_index), 'get_incoming_links'), (undefined(name='anchor') if l_0_anchor is missing else l_0_anchor))
    context.vars['incoming_links'] = l_0_incoming_links
    context.exported_vars.add('incoming_links')
    if ((not t_2((undefined(name='incoming_links') if l_0_incoming_links is missing else l_0_incoming_links))) and (t_1((undefined(name='incoming_links') if l_0_incoming_links is missing else l_0_incoming_links)) > 0)):
        pass
        yield '<template>\n      Incoming link'
        if (t_1((undefined(name='incoming_links') if l_0_incoming_links is missing else l_0_incoming_links)) > 1):
            pass
            yield 's'
        yield ' from:'
        for l_1_incoming_link in (undefined(name='incoming_links') if l_0_incoming_links is missing else l_0_incoming_links):
            l_1_document_type = resolve('document_type')
            l_1_incoming_link_parent_node = l_1_incoming_link_href = missing
            _loop_vars = {}
            pass
            l_1_incoming_link_parent_node = context.call(environment.getattr(l_1_incoming_link, 'parent_node'), _loop_vars=_loop_vars)
            _loop_vars['incoming_link_parent_node'] = l_1_incoming_link_parent_node
            l_1_incoming_link_href = context.call(environment.getattr((undefined(name='link_renderer') if l_0_link_renderer is missing else l_0_link_renderer), 'render_node_link'), context.call(environment.getattr(l_1_incoming_link, 'parent_node'), _loop_vars=_loop_vars), environment.getattr((undefined(name='anchor') if l_0_anchor is missing else l_0_anchor), 'parent_or_including_document'), (undefined(name='document_type') if l_1_document_type is missing else l_1_document_type), _loop_vars=_loop_vars)
            _loop_vars['incoming_link_href'] = l_1_incoming_link_href
            yield '<a href="'
            yield escape((undefined(name='incoming_link_href') if l_1_incoming_link_href is missing else l_1_incoming_link_href))
            yield '">\n        '
            yield escape(context.call(environment.getattr((undefined(name='incoming_link_parent_node') if l_1_incoming_link_parent_node is missing else l_1_incoming_link_parent_node), 'get_display_title'), _loop_vars=_loop_vars))
            yield '\n      </a>'
        l_1_incoming_link = l_1_incoming_link_parent_node = l_1_document_type = l_1_incoming_link_href = missing
        yield '</template>'
    yield '</sdoc-anchor>\n\n'

blocks = {}
debug_info = '8=28&9=32&10=38&11=41&13=44&14=48&15=53&16=55&17=58&18=60'