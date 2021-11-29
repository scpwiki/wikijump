<!DOCTYPE html
    PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN"
    "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="{{$site->getLanguage()}}" lang="{{$site->getLanguage()}}">

<head>
    <title>Wikijump</title>
    <script type="text/javascript" src="/common--javascript/jquery-1.3.2.min.js"></script>
    <script type="text/javascript">
        $j = jQuery.noConflict();
    </script>
    <script type="text/javascript" src="/common--javascript/json.js"></script>

    <script type="text/javascript" src="/common--javascript/combined.js"></script>

    <script type="text/javascript" src="/common--dist/bundle.js"></script>

    <script  type="text/javascript">
        // global request information
            var WIKIREQUEST = {};
            WIKIREQUEST.info = {};
            WIKIREQUEST.info.domain = "{{$site->getDomain()}}";
            WIKIREQUEST.info.siteId = {{$site->getSiteId()}};
            WIKIREQUEST.info.categoryId = {{$category->getCategoryId()}};
            WIKIREQUEST.info.themeId = {{$theme->getThemeId()}};
            WIKIREQUEST.info.requestPageName = "{{$wikiPageName}}";
            OZONE.request.timestamp = {{ time() }};
            OZONE.request.date = new Date();
            WIKIREQUEST.info.lang = '{{$site->getLanguage()}}';
            @if ($wikiPage != null)
            WIKIREQUEST.info.pageUnixName = "{{$wikiPage->getUnixName()}}";
            WIKIREQUEST.info.pageId = {{$wikiPage->getPageId()}};
            @endif
            WIKIREQUEST.info.lang = "{{$site->getLanguage()}}";
            OZONE.lang = "{{$site->getLanguage()}}";
            var HTTP_SCHEMA = '{{$HTTP_SCHEMA}}';
            var URL_HOST = '{{$URL_HOST}}';
            var URL_DOMAIN = '{{$URL_DOMAIN}}';
    </script>


    @if (isset($useCustomDomainScript))
    <script type="text/javascript" src="http@if(isset($useCustomDomainScriptSecure))s@endif://{{$URL_HOST}}/default__flow/Login__CustomDomainScript?site_id={{$site->getSiteId()}}"></script>
    @endif

    @if(isset($usePrivateWikiScript))
    <script type="text/javascript" src="{{$privateWikiScriptUrl}}"></script>
    @endif


    <meta http-equiv="content-type" content="text/html;charset=UTF-8"/>
    <meta http-equiv="content-language" content="{{$site->getLanguage()}}"/>
    <meta name="csrf-token" content="{{ csrf_token() }}"/>
    <style type="text/css" id="internal-style">

        @foreach ($theme->getStyleUrls() as $file)
        @import url({{$file}});
        @endforeach

    </style>

    <link rel="shortcut icon" href="/common--theme/base/images/favicon.gif"/>
    <link rel="icon" type="image/gif" href="/common--theme/base/images/favicon.gif"/>

</head>

<body id="html-body">

<div id="container-wrap">
    <div id="container">
        <div id="header">
            <h1><a href="/"><span>{{$site->getName()}}</span></a></h1>
            @if($site->getSubtitle() != null)
            <h2><span>{{$site->getSubtitle()}}</span></h2>
            @endif

            <div id="search-top-box">
                <form id="search-top-box-form" action="dummy">
                    <input id="search-top-box-input" class="text empty" type="text" size="15" name="query" value="{{_('search this wiki')}}" onfocus="if(YAHOO.util.Dom.hasClass(this, 'empty')){YAHOO.util.Dom.removeClass(this,'empty'); this.value='';}"/><input class="button" type="submit" name="search" value="{{_('search')}}"/>
                </form>
            </div>

            @if ($topBarContent != null)
            <div id="top-bar">
                {!! $topBarContent !!}
            </div>
            @endif
            <div id="login-status">
                @include('legacy.loginstatus')
            </div>
            <div id="header-extra-div-1"><span></span></div><div id="header-extra-div-2"><span></span></div><div id="header-extra-div-3"><span></span></div>
        </div>

        <div id="content-wrap">
            @if ($sideBar1Content != null)
            <div id="side-bar">
                {!! $sideBar1Content !!}
            </div>
            @endif

            <div id="main-content">
                <div id="action-area-top"></div>

                @if ($wikiPage == null || $wikiPage->getTitle() != '')
                <div id="page-title">
                    @if($wikiPage != null)
                    {{$wikiPage->getTitle()}}
                    @else
                        The page does not (yet) exist.
                    @endif
                </div>
                @endif
                @if ($breadcrumbs != null)
                <div id="breadcrumbs">
                    @foreach ($breadcrumbs as $breadcrumb)
                    <a href="/{{$breadcrumb->getUnixName()}}">{{$breadcrumb->getTitle()}}</a> &raquo;
                    @endforeach
                    {{$wikiPage->getTitleOrUnixName()}}
                </div>
                @endif


                <div id="page-content">
                    @yield('content')
                </div>
                @if (is_countable($tags))
                <div class="page-tags">
							<span>
								page tags:
                                @foreach ($tags as $tag)
									<a href="/system:page-tags/tag/{$tag|escape:'url'}#pages">{{$tag}}</a>
                                @endforeach
							</span>
                </div>
                @endif

                <div style="clear:both; height:1px; font-size:1px;"></div>
                @if (!isset($pageNotExists))
                    {!! $pageOptions !!}
                @endif

                <div id="action-area" style="display: none"></div>
            </div>
        </div>

        <div id="footer">
            <div class="options">
                <a href="http://www.wikijump.com/docs" id="wikijump-help-button">
                    help
                </a>
                <a href="http://www.wikijump.com/legal:terms-of-service"   id="wikijump-tos-button">
                    terms of service
                </a>
                |
                <a href="http://www.wikijump.com/legal:privacy-policy"   id="wikijump-privacy-button">
                    privacy
                </a>
                |
                <a href="javascript:;" id="bug-report-button"
                   onclick="Wikijump.page.listeners.pageBugReport(event)">
                    report a bug
                </a>
            </div>
            @if ($SERVICE_NAME != "")
            Part of <a href="{{$HTTP_SCHEMA}}://{{$URL_HOST}}">{{$SERVICE_NAME}}</a>
            &#8212;
            @endif
            Powered by <a href="https://github.com/scpwiki/wikijump">Wikijump</a>
        </div>
        <div id="license-area" class="license-area">
            {!! $licenseHtml !!}
        </div>

        <div id="extrac-div-1"><span></span></div><div id="extrac-div-2"><span></span></div><div id="extrac-div-3"><span></span></div>

    </div>
</div>

<!-- These extra divs/spans may be used as catch-alls to add extra imagery. -->
<div id="extra-div-1"><span></span></div><div id="extra-div-2"><span></span></div><div id="extra-div-3"><span></span></div>
<div id="extra-div-4"><span></span></div><div id="extra-div-5"><span></span></div><div id="extra-div-6"><span></span></div>


<div id="page-options-bottom-tips" style="display: none">
    <div id="edit-button-hovertip">
        Click here to edit contents of this page.
    </div>
</div>
<div id="page-options-bottom-2-tips"  style="display: none">
    <div id="edit-sections-button-hovertip">
        Click here to toggle editing of individual sections of the page (if possible).
        Watch headings for an "edit" link when available.
    <div id="history-button-hovertip">
        Check out how this page has evolved in the past.
    </div>
    <div id="discuss-button-hovertip">
        If you want to discuss contents of this page - this is the easiest way to do it.
    </div>
    <div id="files-button-hovertip">
        View and manage file attachments for this page.
    </div>
    <div id="site-tools-button-hovertip">
        A few useful tools to manage this Site.
    </div>
    <div id="backlinks-button-hovertip">
        See pages that link to and include this page.
    </div>
    <div id="rename-move-button-hovertip">
        Change the name (also URL address, possibly the category) of the page.
    </div>
    <div id="view-source-button-hovertip">
        View wiki source for this page without editing.
    </div>
    <div id="parent-page-button-hovertip">
        View/set parent page (used for creating breadcrumbs and structured layout).
    </div>
    <div id="bug-report-button-hovertip">
        Something does not work as expected? Find out what you can do.
    </div>
</div>

<div id="account-notifications-dummy" style="display:none"></div>

<div style="display:none" id="dummy-ondomready-block"></div>
</body>

</html>
