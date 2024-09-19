Option Explicit

' API URL for your Rust microservice
Private Const RUST_API_URL As String = "http://localhost:8000/api"

' Function to call Rust API
Public Function CallRustAPI(endpoint As String, Optional method As String = "GET", Optional data As String = "") As String
    Dim request As Object
    Set request = CreateObject("MSXML2.XMLHTTP")
    
    On Error GoTo ErrorHandler
    
    request.Open method, RUST_API_URL & "/" & endpoint, False
    request.setRequestHeader "Content-Type", "application/json"
    
    If method = "POST" Then
        request.send data
    Else
        request.send
    End If
    
    CallRustAPI = request.responseText
    Exit Function
    
ErrorHandler:
    CallRustAPI = "Error: " & Err.Description
End Function

' Function to check if data is ready
Public Function IsDataReady() As Boolean
    Dim response As String
    response = CallRustAPI("status")
    IsDataReady = (InStr(response, "ready") > 0)
End Function

' Sub to initiate data retrieval
Public Sub RequestData()
    Application.StatusBar = "Requesting data from Rust microservice..."
    CallRustAPI "request", "POST", "{""action"": ""fetch_data""}"
    Application.OnTime Now + TimeValue("00:00:01"), "CheckDataStatus"
End Sub

' Sub to check data status
Public Sub CheckDataStatus()
    If IsDataReady() Then
        ProcessData
    Else
        Application.StatusBar = "Waiting for data..."
        Application.OnTime Now + TimeValue("00:00:01"), "CheckDataStatus"
    End If
End Sub

' Sub to process received data
Private Sub ProcessData()
    Dim data As String
    data = CallRustAPI("data")
    
    ' Process the data here (e.g., parse JSON, update cells)
    ' For example:
    ' Sheet1.Range("A1").Value = ParseJsonValue(data, "some_field")
    
    Application.StatusBar = "Data processed successfully!"
End Sub