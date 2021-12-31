{{--
    Dashboard view, which is an SPA on the frontend.
--}}

@extends('next.frame', ['plain' => true, 'title' => __('dashboard')])

@section('content')
    <wj-component-loader ld-load="Dashboard" ld-skeleton="spinner:16rem">
    </wj-component-loader>
@endsection
