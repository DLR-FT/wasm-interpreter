from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/document/table/main.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_view_object = resolve('view_object')
    pass
    if context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'has_any_nodes')):
        pass
        yield '<div class="main">\n    '
        template = environment.get_template('_shared/tags.jinja.html', 'screens/document/table/main.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n      <div class="content">\n        <sdoc-node>\n        '
        template = environment.get_template('components/node_field/document_title/index.jinja', 'screens/document/table/main.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n        '
        template = environment.get_template('components/node_field/document_meta/index.jinja', 'screens/document/table/main.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n\n        </sdoc-node>\n        <table class="content-view-table">\n          <thead>\n            <tr>\n              <th class="content-view-th">Type</th>\n              <th class="content-view-th">Level</th>'
        for l_1_meta_field_title in context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'enumerate_meta_field_titles')):
            _loop_vars = {}
            pass
            yield '<th class="content-view-th">'
            yield escape(l_1_meta_field_title)
            yield '</th>'
        l_1_meta_field_title = missing
        yield '<th class="content-view-th">REFS</th>\n              <th class="content-view-th">Title</th>\n              <th class="content-view-th">Statement</th>\n              <th class="content-view-th">Rationale</th>\n              <th class="content-view-th">Comment</th>'
        for l_1_meta_field_title in context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'enumerate_custom_content_field_titles')):
            _loop_vars = {}
            pass
            yield '<th class="content-view-th">'
            yield escape(l_1_meta_field_title)
            yield '</th>'
        l_1_meta_field_title = missing
        yield '</tr>\n          </thead>'
        for (l_1_node, l_1__) in context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document_content_iterator')):
            l_1_requirement = resolve('requirement')
            l_1_requirement_file_links = resolve('requirement_file_links')
            l_1_section = resolve('section')
            _loop_vars = {}
            pass
            if context.call(environment.getattr(l_1_node, 'is_requirement'), _loop_vars=_loop_vars):
                pass
                l_1_requirement = l_1_node
                _loop_vars['requirement'] = l_1_requirement
                yield '\n              <tr>\n                <td class="content-view-td content-view-td-type">'
                if context.call(environment.getattr(l_1_node, 'is_text_node'), _loop_vars=_loop_vars):
                    pass
                    yield '\n                    Text'
                else:
                    pass
                    yield '\n                    Requirement'
                yield '</td>\n                <td class="content-view-td content-view-td-meta">\n                  '
                yield escape(environment.getattr(environment.getattr((undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), 'context'), 'title_number_string'))
                yield '\n                </td>'
                for l_2_meta_field_title in context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'enumerate_meta_field_titles'), _loop_vars=_loop_vars):
                    l_2_field_value = missing
                    _loop_vars = {}
                    pass
                    l_2_field_value = context.call(environment.getattr((undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), 'get_meta_field_value_by_title'), l_2_meta_field_title, _loop_vars=_loop_vars)
                    _loop_vars['field_value'] = l_2_field_value
                    yield '\n                  <td class="content-view-td content-view-td-meta">'
                    if (undefined(name='field_value') if l_2_field_value is missing else l_2_field_value):
                        pass
                        yield escape((undefined(name='field_value') if l_2_field_value is missing else l_2_field_value))
                    yield '</td>'
                l_2_meta_field_title = l_2_field_value = missing
                yield '<td class="content-view-td content-view-td-meta content-view-td-related">'
                if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'traceability_index'), 'has_parent_requirements'), (undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), _loop_vars=_loop_vars):
                    pass
                    yield '\n                    Parents:\n                    <ul class="requirement__link">'
                    for l_2_requirement in context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'traceability_index'), 'get_parent_requirements'), (undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), _loop_vars=_loop_vars):
                        _loop_vars = {}
                        pass
                        yield '\n                        <li>\n                          <a class="requirement__link-parent"\n                            href="'
                        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_node_link'), l_2_requirement, _loop_vars=_loop_vars))
                        yield '">'
                        if environment.getattr(l_2_requirement, 'reserved_uid'):
                            pass
                            yield '\n                              <span class="requirement__parent-uid">'
                            yield escape(environment.getattr(l_2_requirement, 'reserved_uid'))
                            yield '</span>'
                        yield '\n                            '
                        yield escape((environment.getattr(l_2_requirement, 'reserved_title') if environment.getattr(l_2_requirement, 'reserved_title') else ''))
                        yield '\n                          </a>\n                        </li>'
                    l_2_requirement = missing
                    yield '\n                    </ul>'
                if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'traceability_index'), 'has_children_requirements'), (undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), _loop_vars=_loop_vars):
                    pass
                    yield '\n                    Children:\n                    <ul class="requirement__link">'
                    for l_2_requirement in context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'traceability_index'), 'get_children_requirements'), (undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), _loop_vars=_loop_vars):
                        _loop_vars = {}
                        pass
                        yield '\n                        <li>\n                          <a class="requirement__link-child"\n                            href="'
                        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_node_link'), l_2_requirement, _loop_vars=_loop_vars))
                        yield '">'
                        if environment.getattr(l_2_requirement, 'reserved_uid'):
                            pass
                            yield '\n                              <span class="requirement__child-uid">'
                            yield escape(environment.getattr(l_2_requirement, 'reserved_uid'))
                            yield '</span>'
                        yield '\n                            '
                        yield escape((environment.getattr(l_2_requirement, 'reserved_title') if environment.getattr(l_2_requirement, 'reserved_title') else ''))
                        yield '\n                          </a>\n                        </li>'
                    l_2_requirement = missing
                    yield '\n                    </ul>'
                if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_activated_requirements_to_source_traceability'), _loop_vars=_loop_vars):
                    pass
                    l_1_requirement_file_links = context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'traceability_index'), 'get_requirement_file_links'), (undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), _loop_vars=_loop_vars)
                    _loop_vars['requirement_file_links'] = l_1_requirement_file_links
                    if (undefined(name='requirement_file_links') if l_1_requirement_file_links is missing else l_1_requirement_file_links):
                        pass
                        yield '\n                      Source files:\n                      <ul class="requirement__link">'
                        for (l_2_link, l_2_markers) in (undefined(name='requirement_file_links') if l_1_requirement_file_links is missing else l_1_requirement_file_links):
                            _loop_vars = {}
                            pass
                            for l_3_marker in l_2_markers:
                                _loop_vars = {}
                                pass
                                yield '\n                            <li>\n                              <a class="requirement__link-file"\n                                  href="'
                                yield escape(context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'link_renderer'), 'render_source_file_link'), (undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), l_2_link, _loop_vars=_loop_vars))
                                yield '#'
                                yield escape(environment.getattr((undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), 'reserved_uid'))
                                yield '#'
                                yield escape(environment.getattr(l_3_marker, 'ng_range_line_begin'))
                                yield '#'
                                yield escape(environment.getattr(l_3_marker, 'ng_range_line_end'))
                                yield '">\n                                '
                                yield escape(l_2_link)
                                yield ', <i>lines: '
                                yield escape(environment.getattr(l_3_marker, 'ng_range_line_begin'))
                                yield '-'
                                yield escape(environment.getattr(l_3_marker, 'ng_range_line_end'))
                                yield '</i>\n                              </a>\n                            </li>'
                            l_3_marker = missing
                        l_2_link = l_2_markers = missing
                        yield '</ul>'
                yield '\n                </td>\n                <td class="content-view-td content-view-td-title">'
                if environment.getattr((undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), 'reserved_title'):
                    pass
                    l_2_sdoc_entity = (undefined(name='requirement') if l_1_requirement is missing else l_1_requirement)
                    pass
                    template = environment.get_template('components/anchor/index.jinja', 'screens/document/table/main.jinja')
                    gen = template.root_render_func(template.new_context(context.get_all(), True, {'sdoc_entity': l_2_sdoc_entity, '_': l_1__, 'node': l_1_node, 'requirement': l_1_requirement, 'requirement_file_links': l_1_requirement_file_links, 'section': l_1_section}))
                    try:
                        for event in gen:
                            yield event
                    finally: gen.close()
                    l_2_sdoc_entity = missing
                    yield '\n                    <div class="requirement__title">'
                    yield escape(environment.getattr((undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), 'reserved_title'))
                    yield '</div>'
                yield '</td>\n                <td class="content-view-td content-view-td-content">'
                if context.call(environment.getattr((undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), 'has_reserved_statement'), _loop_vars=_loop_vars):
                    pass
                    yield '<sdoc-autogen>'
                    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_node_statement'), (undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), _loop_vars=_loop_vars))
                    yield '</sdoc-autogen>'
                yield '</td>\n                <td class="content-view-td content-view-td-content">'
                if environment.getattr((undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), 'rationale'):
                    pass
                    yield '<sdoc-autogen>'
                    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_node_rationale'), (undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), _loop_vars=_loop_vars))
                    yield '</sdoc-autogen>'
                yield '</td>\n                <td class="content-view-td content-view-td-content">'
                for l_2_comment_field_ in context.call(environment.getattr((undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), 'get_comment_fields'), _loop_vars=_loop_vars):
                    _loop_vars = {}
                    pass
                    yield '<sdoc-autogen>'
                    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_node_field'), l_2_comment_field_, _loop_vars=_loop_vars))
                    yield '</sdoc-autogen>'
                l_2_comment_field_ = missing
                yield '</td>'
                for l_2_meta_field_title in context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'enumerate_custom_content_field_titles'), _loop_vars=_loop_vars):
                    l_2_field_value = missing
                    _loop_vars = {}
                    pass
                    l_2_field_value = context.call(environment.getattr((undefined(name='requirement') if l_1_requirement is missing else l_1_requirement), 'get_meta_field_value_by_title'), l_2_meta_field_title, _loop_vars=_loop_vars)
                    _loop_vars['field_value'] = l_2_field_value
                    yield '\n                  <td class="content-view-td">'
                    if (undefined(name='field_value') if l_2_field_value is missing else l_2_field_value):
                        pass
                        yield '\n                      '
                        yield escape((undefined(name='field_value') if l_2_field_value is missing else l_2_field_value))
                    yield '</td>'
                l_2_meta_field_title = l_2_field_value = missing
                yield '</tr>'
            elif context.call(environment.getattr(l_1_node, 'is_section'), _loop_vars=_loop_vars):
                pass
                l_1_section = l_1_node
                _loop_vars['section'] = l_1_section
                yield '\n              <tr>\n                <td class="content-view-td content-view-td-type">Section</td>\n                <td class="content-view-td content-view-td-meta">'
                yield escape(environment.getattr(environment.getattr((undefined(name='section') if l_1_section is missing else l_1_section), 'context'), 'title_number_string'))
                yield '</td>'
                for l_2_meta_field_title in context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'enumerate_meta_field_titles'), _loop_vars=_loop_vars):
                    _loop_vars = {}
                    pass
                    yield '<td class="content-view-td content-view-td-meta"></td>'
                l_2_meta_field_title = missing
                yield '<td class="content-view-td content-view-td-meta"></td>\n                <td class="content-view-td content-view-td-title" colspan="4">'
                if environment.getattr((undefined(name='section') if l_1_section is missing else l_1_section), 'title'):
                    pass
                    l_2_sdoc_entity = (undefined(name='section') if l_1_section is missing else l_1_section)
                    pass
                    template = environment.get_template('components/anchor/index.jinja', 'screens/document/table/main.jinja')
                    gen = template.root_render_func(template.new_context(context.get_all(), True, {'sdoc_entity': l_2_sdoc_entity, '_': l_1__, 'node': l_1_node, 'requirement': l_1_requirement, 'requirement_file_links': l_1_requirement_file_links, 'section': l_1_section}))
                    try:
                        for event in gen:
                            yield event
                    finally: gen.close()
                    l_2_sdoc_entity = missing
                    yield '\n                    <div class="requirement__title">\n                      '
                    yield escape(environment.getattr((undefined(name='section') if l_1_section is missing else l_1_section), 'title'))
                    yield '\n                    </div>'
                    for l_2_meta_field_title in context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'enumerate_custom_content_field_titles'), _loop_vars=_loop_vars):
                        _loop_vars = {}
                        pass
                        yield '<td class="content-view-td">\n                      </td>'
                    l_2_meta_field_title = missing
                yield '</td>\n              </tr>'
        l_1_node = l_1__ = l_1_requirement = l_1_requirement_file_links = l_1_section = missing
        yield '\n        </table>\n      </div>\n      \n  </div>\n  '
    else:
        pass
        yield '<sdoc-main-placeholder data-testid="document-main-placeholder">\n  This view is empty because\n  <br />\n  the document has no content.\n  </sdoc-main-placeholder>'

blocks = {}
debug_info = '1=12&3=15&6=22&7=29&15=36&16=40&23=44&24=48&28=52&29=58&30=60&33=63&40=70&42=72&43=76&45=79&46=81&51=85&54=88&57=92&58=94&59=97&61=100&67=104&70=107&73=111&74=113&75=116&77=119&83=123&84=125&85=127&88=130&89=133&92=137&93=145&103=155&105=159&107=167&111=170&112=173&116=176&117=179&121=182&122=186&125=190&126=194&128=197&129=200&134=204&135=206&138=209&139=211&144=217&146=221&149=229&151=231'