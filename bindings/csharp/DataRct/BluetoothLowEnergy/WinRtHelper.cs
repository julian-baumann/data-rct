using System.Runtime.CompilerServices;
using System.Runtime.InteropServices.WindowsRuntime;
using Windows.Foundation;
using Windows.Storage.Streams;

static class WinRtHelper
{
    struct VoidValueTypeParameter { }

    public static TaskAwaiter GetAwaiter(this IAsyncAction op)
    {
        var tcs = new TaskCompletionSource<VoidValueTypeParameter>();

        op.Completed = (IAsyncAction asyncStatus, AsyncStatus unused) =>
        {
            switch (asyncStatus.Status)
            {
                case AsyncStatus.Canceled:
                    tcs.SetCanceled();
                    break;
                case AsyncStatus.Error:
                    tcs.SetException(asyncStatus.ErrorCode);
                    break;
                case AsyncStatus.Completed:
                    tcs.SetResult(default(VoidValueTypeParameter));
                    break;
            }
        };

        Task t = tcs.Task;
        return t.GetAwaiter();
    }

    public static TaskAwaiter<T> GetAwaiter<T>(this IAsyncOperation<T> op)
    {
        var tcs = new TaskCompletionSource<T>();

        op.Completed = (IAsyncOperation<T> asyncStatus, AsyncStatus unused) =>
        {
            switch (asyncStatus.Status)
            {
                case AsyncStatus.Canceled:
                    tcs.SetCanceled();
                    break;
                case AsyncStatus.Error:
                    tcs.SetException(asyncStatus.ErrorCode);
                    break;
                case AsyncStatus.Completed:
                    tcs.SetResult(asyncStatus.GetResults());
                    break;
            }
        };

        return tcs.Task.GetAwaiter();
    }
    
    
    
    
    public static TaskAwaiter GetAwaiter<P>(this IAsyncActionWithProgress<P> op)
    {
        var tcs = new TaskCompletionSource<VoidValueTypeParameter>();

        op.Completed = (IAsyncActionWithProgress<P> asyncStatus, AsyncStatus unused) =>
        {
            switch (asyncStatus.Status)
            {
                case AsyncStatus.Canceled:
                    tcs.SetCanceled();
                    break;
                case AsyncStatus.Error:
                    tcs.SetException(asyncStatus.ErrorCode);
                    break;
                case AsyncStatus.Completed:
                    tcs.SetResult(default(VoidValueTypeParameter));
                    break;
            }
        };

        Task t = tcs.Task;
        return t.GetAwaiter();
    }

    public static TaskAwaiter<T> GetAwaiter<T, P>(this IAsyncOperationWithProgress<T, P> op)
    {
        var tcs = new TaskCompletionSource<T>();

        op.Completed = (IAsyncOperationWithProgress<T, P> asyncStatus, AsyncStatus unused) =>
        {
            switch (asyncStatus.Status)
            {
                case AsyncStatus.Canceled:
                    tcs.SetCanceled();
                    break;
                case AsyncStatus.Error:
                    tcs.SetException(asyncStatus.ErrorCode);
                    break;
                case AsyncStatus.Completed:
                    tcs.SetResult(asyncStatus.GetResults());
                    break;
            }
        };

        return tcs.Task.GetAwaiter();
    }
}