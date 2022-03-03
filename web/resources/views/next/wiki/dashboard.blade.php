{{--
    Dashboard view, which is an SPA on the frontend.
--}}

@extends('next.frame', ['plain' => true, 'title' => __('dashboard')])

@push('preloads')
    @preload('dashboard.scss')
@endpush

@push('styles')
    @vite('dashboard.scss')
@endpush


@section('content')
    <wj-component-loader ld-load="Dashboard" ld-skeleton="spinner:16rem">
    </wj-component-loader>
@endsection
