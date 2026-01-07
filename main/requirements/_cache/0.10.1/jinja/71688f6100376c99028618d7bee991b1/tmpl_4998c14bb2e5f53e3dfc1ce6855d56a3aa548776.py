from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/project_index/main.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_view_object = resolve('view_object')
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
    yield '<div class="main">\n  <div class="dashboard">\n    <div class="dashboard-main">\n      '
    template = environment.get_template('screens/project_index/frame_project_tree.jinja.html', 'screens/project_index/main.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '\n    </div>\n    <div class="dashboard-aside">\n      '
    if context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'should_display_fragments_toggle')):
        pass
        yield '\n      <div class="dashboard-block" id="project_tree_controls"></div>\n      '
    yield '\n\n      <div class="dashboard-block">\n        \n        \n        <div class="dashboard-block-title">\n          Project tree configuration\n        </div>\n\n        '
    if t_1(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'input_paths')):
        pass
        yield '\n          <b>Input paths:</b>\n          '
        for l_1_path_ in environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'input_paths'):
            _loop_vars = {}
            pass
            yield '\n          <code style="word-wrap: break-word;">'
            yield escape(l_1_path_)
            yield '</code><br/>\n          '
        l_1_path_ = missing
        yield '\n        '
    yield '\n\n        '
    if (t_1(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'include_doc_paths')) or t_1(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'exclude_doc_paths'))):
        pass
        yield '\n          <p>Document paths:</p>\n          <ul>\n            '
        for l_1_path_ in environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'include_doc_paths'):
            _loop_vars = {}
            pass
            yield '\n            <li style="list-style-type: \'✔️    \'">'
            yield escape(l_1_path_)
            yield '</li>\n            '
        l_1_path_ = missing
        yield '\n            '
        for l_1_path_ in environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'exclude_doc_paths'):
            _loop_vars = {}
            pass
            yield '\n            <li style="list-style-type: \'⛔    \'">'
            yield escape(l_1_path_)
            yield '</li>\n            '
        l_1_path_ = missing
        yield '\n          </ul>\n        '
    yield '\n\n        '
    if (not t_2(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'source_root_path'))):
        pass
        yield '\n          <b>Source root path:</b>\n          <div>\n            <code style="word-wrap: break-word;">'
        yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'source_root_path'))
        yield '</code>\n          </div>\n        '
    yield '\n\n        '
    if (t_1(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'include_source_paths')) or t_1(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'exclude_source_paths'))):
        pass
        yield '\n          <p>Source paths:</p>\n          <ul>\n            '
        for l_1_path_ in environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'include_source_paths'):
            _loop_vars = {}
            pass
            yield '\n            <li style="list-style-type: \'✔️    \'">'
            yield escape(l_1_path_)
            yield '</li>\n            '
        l_1_path_ = missing
        yield '\n            '
        for l_1_path_ in environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'exclude_source_paths'):
            _loop_vars = {}
            pass
            yield '\n            <li style="list-style-type: \'⛔    \'">'
            yield escape(l_1_path_)
            yield '</li>\n            '
        l_1_path_ = missing
        yield '\n          </ul>\n        '
    yield '\n\n      </div>\n    </div>\n  </div>\n</div>'

blocks = {}
debug_info = '4=25&7=32&18=36&20=39&21=43&25=48&28=51&29=55&31=59&32=63&37=68&40=71&44=74&47=77&48=81&50=85&51=89'