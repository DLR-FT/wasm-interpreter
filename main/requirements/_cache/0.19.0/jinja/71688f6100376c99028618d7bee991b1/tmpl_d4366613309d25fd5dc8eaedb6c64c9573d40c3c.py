from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/project_index/index.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    parent_template = None
    l_0_template_type = missing
    pass
    l_0_template_type = 'Project index'
    context.vars['template_type'] = l_0_template_type
    context.exported_vars.add('template_type')
    if parent_template is None:
        yield '\n\n'
    parent_template = environment.get_template('base.jinja.html', 'screens/project_index/index.jinja')
    for name, parent_block in parent_template.blocks.items():
        context.blocks.setdefault(name, []).append(parent_block)
    yield from parent_template.root_render_func(context)

def block_head_css(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_super = context.super('head_css', block_head_css)
    _block_vars = {}
    l_0_view_object = resolve('view_object')
    pass
    yield '\n  '
    yield escape(context.call(l_0_super, _block_vars=_block_vars))
    yield '\n  <link rel="stylesheet" href="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'project_tree.css', _block_vars=_block_vars))
    yield '"/>\n'

def block_head_scripts(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    l_0_view_object = resolve('view_object')
    pass
    yield '\n  <script src="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'stimulus_umd.min.js', _block_vars=_block_vars))
    yield '"></script>\n  <script>\n    Stimulus.application = Stimulus.Application.start();\n  </script>\n  \n  <script src="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'app_core.js', _block_vars=_block_vars))
    yield '"></script>\n\n  <script src="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'collapsible_tree.js', _block_vars=_block_vars))
    yield '"></script>\n  <script src="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'stable_uri_forwarder.js', _block_vars=_block_vars))
    yield '" id="stable_uri_forwarder"></script>\n\n  \n  <script src="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'project_tree.js', _block_vars=_block_vars))
    yield '"></script>\n\n  '
    if True:
        pass
        yield '\n    '
        template = environment.get_template('_shared/static_search_head.jinja', 'screens/project_index/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n  '
    if (environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_running_on_server') and (not environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'standalone'))):
        pass
        yield '<script type="module">\n    import hotwiredTurbo from "'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url_with_prefix'), 'turbo.min.js', _block_vars=_block_vars))
        yield '";\n  </script>\n  <script src="'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'dropdown_menu.js', _block_vars=_block_vars))
        yield '"></script>\n  <script src="'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'controllers/editable_field_controller.js', _block_vars=_block_vars))
        yield '"></script>\n  <script src="'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'controllers/modal_controller.js', _block_vars=_block_vars))
        yield '"></script>'

def block_title(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    l_0_view_object = resolve('view_object')
    l_0_template_type = resolve('template_type')
    pass
    yield '\n  '
    yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'project_title'))
    yield ' - '
    yield escape((undefined(name='template_type') if l_0_template_type is missing else l_0_template_type))
    yield '\n'

def block_viewtype(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    pass
    yield 'document-tree'

def block_layout_nav(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    pass
    yield '\n  '
    template = environment.get_template('_shared/nav.jinja.html', 'screens/project_index/index.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '\n'

def block_toc_content(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    pass
    yield '\n'

def block_header_content(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    l_0_template_type = resolve('template_type')
    pass
    l_1_header__pagetype = (undefined(name='template_type') if l_0_template_type is missing else l_0_template_type)
    l_1_header__last = 'screens/project_index/actions.jinja'
    pass
    template = environment.get_template('components/header/index.jinja', 'screens/project_index/index.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {'header__last': l_1_header__last, 'header__pagetype': l_1_header__pagetype}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    l_1_header__pagetype = l_1_header__last = missing

def block_main_content(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    pass
    yield '\n  '
    template = environment.get_template('screens/project_index/main.jinja', 'screens/project_index/index.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '\n'

blocks = {'head_css': block_head_css, 'head_scripts': block_head_scripts, 'title': block_title, 'viewtype': block_viewtype, 'layout_nav': block_layout_nav, 'toc_content': block_toc_content, 'header_content': block_header_content, 'main_content': block_main_content}
debug_info = '1=13&3=18&5=23&6=34&7=36&10=39&11=49&16=51&18=53&19=55&28=57&30=59&31=62&34=69&36=72&38=74&39=76&40=78&44=81&45=92&48=97&50=107&51=116&54=124&57=134&62=146&66=154&67=163'