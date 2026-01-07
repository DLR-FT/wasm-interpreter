from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/source_file_view/main.jinja'

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
    pass
    yield '<div class="main">\n\n<div id="sourceContainer" class="source-file__source">\n\n  '
    template = environment.get_template('screens/source_file_view/file_stats.jinja', 'screens/source_file_view/main.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    if (t_1(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'pygmented_source_file_lines')) > 0):
        pass
        yield '<div id="source" class="source">'
        l_1_loop = missing
        for l_1_line, l_1_loop in LoopContext(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'pygmented_source_file_lines'), undefined):
            l_1_current_marker_link = resolve('current_marker_link')
            l_1_current_range_begin = resolve('current_range_begin')
            l_1_current_range_end = resolve('current_range_end')
            _loop_vars = {}
            pass
            if ((environment.getattr(environment.getattr(l_1_line, '__class__'), '__name__') == 'SourceMarkerTuple') and (not context.call(environment.getattr(l_1_line, 'is_end'), _loop_vars=_loop_vars))):
                pass
                l_1_current_marker_link = context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_marker_range_link'), l_1_line, _loop_vars=_loop_vars)
                _loop_vars['current_marker_link'] = l_1_current_marker_link
                l_1_current_range_begin = environment.getattr(l_1_line, 'ng_range_line_begin')
                _loop_vars['current_range_begin'] = l_1_current_range_begin
                l_1_current_range_end = environment.getattr(l_1_line, 'ng_range_line_end')
                _loop_vars['current_range_end'] = l_1_current_range_end
                yield '<div\n          class="source__range collapsed"\n          data-begin="'
                yield escape((undefined(name='current_range_begin') if l_1_current_range_begin is missing else l_1_current_range_begin))
                yield '"\n          data-end="'
                yield escape((undefined(name='current_range_end') if l_1_current_range_end is missing else l_1_current_range_end))
                yield '"\n        >\n          <div class="source__range-header">'
                l_2_begin = (undefined(name='current_range_begin') if l_1_current_range_begin is missing else l_1_current_range_begin)
                l_2_end = (undefined(name='current_range_end') if l_1_current_range_end is missing else l_1_current_range_end)
                l_2_href = (undefined(name='current_marker_link') if l_1_current_marker_link is missing else l_1_current_marker_link)
                l_2_scope = context.call(environment.getattr(environment.getitem(environment.getattr(l_1_line, 'markers'), 0), 'get_description'), _loop_vars=_loop_vars)
                pass
                template = environment.get_template('screens/source_file_view/range_button.jinja', 'screens/source_file_view/main.jinja')
                gen = template.root_render_func(template.new_context(context.get_all(), True, {'begin': l_2_begin, 'end': l_2_end, 'href': l_2_href, 'scope': l_2_scope, 'current_marker_link': l_1_current_marker_link, 'current_range_begin': l_1_current_range_begin, 'current_range_end': l_1_current_range_end, 'line': l_1_line, 'loop': l_1_loop}))
                try:
                    for event in gen:
                        yield event
                finally: gen.close()
                l_2_begin = l_2_end = l_2_href = l_2_scope = missing
                yield '</div>\n          <div class="source__range-cell">\n            <div\n              class="source__range-handler"\n              data-begin="'
                yield escape((undefined(name='current_range_begin') if l_1_current_range_begin is missing else l_1_current_range_begin))
                yield '"\n              data-end="'
                yield escape((undefined(name='current_range_end') if l_1_current_range_end is missing else l_1_current_range_end))
                yield '"\n            >'
                template = environment.get_template('_res/svg_ico16_section_collapse.jinja', 'screens/source_file_view/main.jinja')
                gen = template.root_render_func(template.new_context(context.get_all(), True, {'current_marker_link': l_1_current_marker_link, 'current_range_begin': l_1_current_range_begin, 'current_range_end': l_1_current_range_end, 'line': l_1_line, 'loop': l_1_loop}))
                try:
                    for event in gen:
                        yield event
                finally: gen.close()
                yield '</div>\n          </div>\n          <div class="source__range-cell">\n            <ul class="source__range-titles-list">'
                for l_2_marker_ in environment.getattr(l_1_line, 'markers'):
                    _loop_vars = {}
                    pass
                    for l_3_req in environment.getattr(l_2_marker_, 'reqs_objs'):
                        _loop_vars = {}
                        pass
                        yield '<li>\n                  '
                        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_node_title_for_banner_header'), l_2_marker_, environment.getattr(l_3_req, 'uid'), _loop_vars=_loop_vars))
                        yield '\n                </li>'
                    l_3_req = missing
                l_2_marker_ = missing
                yield '</ul>\n            <div class="source__range-banner source__range-start">'
                for l_2_marker_ in environment.getattr(l_1_line, 'markers'):
                    _loop_vars = {}
                    pass
                    for l_3_req in environment.getattr(l_2_marker_, 'reqs_objs'):
                        _loop_vars = {}
                        pass
                        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_detailed_node_for_banner'), environment.getattr(l_3_req, 'uid'), _loop_vars=_loop_vars))
                    l_3_req = missing
                l_2_marker_ = missing
                yield '</div>\n          </div>\n        </div>\n      '
            yield '\n\n      <div data-line="'
            yield escape(environment.getattr(l_1_loop, 'index'))
            yield '" class="source__line">\n        <div data-line="'
            yield escape(environment.getattr(l_1_loop, 'index'))
            yield '" id="line-'
            yield escape(environment.getattr(l_1_loop, 'index'))
            yield '" class="source__line-number"><pre>'
            yield escape(environment.getattr(l_1_loop, 'index'))
            yield '</pre></div>\n        <div data-line="'
            yield escape(environment.getattr(l_1_loop, 'index'))
            yield '" class="source__line-content">'
            if (environment.getattr(environment.getattr(l_1_line, '__class__'), '__name__') == 'SourceMarkerTuple'):
                pass
                yield '\n            <pre class="highlight">'
                yield escape(environment.getattr(l_1_line, 'source_line'))
                yield '</pre>'
            elif (l_1_line != ''):
                pass
                yield '\n            <pre class="highlight">'
                yield escape(l_1_line)
                yield '</pre>'
            else:
                pass
                yield '<pre data-state="empty" style="user-select: none">&nbsp;</pre>'
            yield '</div>\n      </div>'
            if ((environment.getattr(environment.getattr(l_1_line, '__class__'), '__name__') == 'SourceMarkerTuple') and (context.call(environment.getattr(l_1_line, 'is_end'), _loop_vars=_loop_vars) or context.call(environment.getattr(l_1_line, 'is_line_marker'), _loop_vars=_loop_vars))):
                pass
                yield '<div\n          class="source__range-closer"\n          data-end="'
                yield escape(environment.getattr(environment.getitem(environment.getattr(l_1_line, 'markers'), 0), 'ng_range_line_end'))
                yield '"\n        >\n          <div class="source__range-closer-label">'
                l_2_begin = environment.getattr(environment.getitem(environment.getattr(l_1_line, 'markers'), 0), 'ng_range_line_begin')
                l_2_end = environment.getattr(environment.getitem(environment.getattr(l_1_line, 'markers'), 0), 'ng_range_line_end')
                l_2_href = context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_marker_range_link'), l_1_line, _loop_vars=_loop_vars)
                l_2_scope = context.call(environment.getattr(environment.getitem(environment.getattr(l_1_line, 'markers'), 0), 'get_description'), _loop_vars=_loop_vars)
                pass
                template = environment.get_template('screens/source_file_view/range_button.jinja', 'screens/source_file_view/main.jinja')
                gen = template.root_render_func(template.new_context(context.get_all(), True, {'begin': l_2_begin, 'end': l_2_end, 'href': l_2_href, 'scope': l_2_scope, 'current_marker_link': l_1_current_marker_link, 'current_range_begin': l_1_current_range_begin, 'current_range_end': l_1_current_range_end, 'line': l_1_line, 'loop': l_1_loop}))
                try:
                    for event in gen:
                        yield event
                finally: gen.close()
                l_2_begin = l_2_end = l_2_href = l_2_scope = missing
                yield '</div>\n        </div>\n      '
        l_1_loop = l_1_line = l_1_current_marker_link = l_1_current_range_begin = l_1_current_range_end = missing
        yield '</div>'
    else:
        pass
        yield '<div style="text-align: center">\n    Source file is empty.\n  </div>'
    yield '</div>\n</div>'

blocks = {}
debug_info = '5=19&7=25&9=29&10=35&11=37&12=39&13=41&16=44&17=46&27=53&34=61&35=63&36=65&40=72&41=75&43=79&49=84&50=87&51=90&59=95&60=97&61=103&62=105&64=108&65=110&67=113&73=119&76=122&85=129'